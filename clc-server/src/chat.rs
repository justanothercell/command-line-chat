use std::collections::HashSet;
use std::ops::ControlFlow;
use std::sync::RwLockWriteGuard;
use futures::SinkExt;
use uuid::Uuid;
use warp::ws::Message;
use clc_lib::protocol::{ChatId, ChatTitle, InviteId, ServerEvent, ServerWsMessage, UserId};
use clc_lib::serialize;
use clc_lib::validator::is_valid_name;
use crate::{Chat, Chats, Client, Clients, debug};

pub(crate) async fn create_chat(title: ChatTitle, user_id: &UserId, clients: &Clients, chats: &Chats){
    if title.len() < 3 || title.len() > 24 {
        send_msg(clients.read().await.get(user_id).unwrap(),
                 ServerWsMessage::SystemMessage(format!("title should be between 3 and 24 characters long, found {}", title.len()))).await;
        return
    }

    if !is_valid_name(&title) {
        send_msg(clients.read().await.get(user_id).unwrap(),
                 ServerWsMessage::SystemMessage(format!("title is not valid"))).await;
        return
    }

    let mut chats_w = chats.write().await;
    let mut clients_w = clients.write().await;
    let mut c = clients_w.get_mut(user_id).unwrap();
    {
        let uuid = Uuid::new_v4().as_simple().to_string();
        debug!("created chat {} {}", title, uuid);
        send_msg(c,ServerWsMessage::SystemEvent(ServerEvent::ChatCreate(uuid.clone(), title.clone()))).await;
        c.chat = Some(uuid.clone());
        chats_w.insert(
            uuid.clone(),
            Chat {
                chat_id: uuid.clone(),
                title,
                owner: user_id.clone(),
                users: HashSet::from([c.user_id.clone()]),
                invites: Default::default()
            },
        );
    };
}

pub(crate) async fn create_chat_invite(user_id: &UserId, clients: &Clients, chats: &Chats){
    let mut chats_w = chats.write().await;
    let clients_w = clients.write().await;
    let user = clients_w.get(user_id).unwrap();
    let chat = chats_w.get_mut(user.chat.as_ref().unwrap()).unwrap();
    if user.user_id != chat.owner {
        send_msg(user, ServerWsMessage::SystemMessage(format!("You have to be admin to create an invite"))).await;
        return;
    }
    let invite = Uuid::new_v4().as_simple().to_string();
    send_msg(user, ServerWsMessage::SystemMessage(format!("Created invite: {}", invite))).await;
    chat.invites.insert(invite);
}

pub(crate) async fn join_chat(user_id: &UserId, chat_title: ChatTitle, invite: InviteId, clients: &Clients, chats: &Chats){
    let mut joined_chat = None;
    let mut chats_w = chats.write().await;
    let chat_exists = chats_w.iter_mut().map(|(id, chat)| {
        if chat.title == chat_title {
            if chat.invites.contains(&invite) {
                chat.invites.remove(&invite);
                chat.users.insert(user_id.to_string());
                joined_chat = Some(chat);
            }
            true
        } else {
            false
        }
    }).any(|x| x);// we use .map().any() instead of just .any() because any short-circuits

    match joined_chat {
        None => {
            let clients_w = clients.write().await;
            let user = clients_w.get(user_id).unwrap();
            if chat_exists {
                send_msg(user, ServerWsMessage::SystemMessage(format!("Your invite id is invalid"))).await;
            } else {
                send_msg(user, ServerWsMessage::SystemMessage(format!("Chat {} does not seem to exist", chat_title))).await;
            }
        }
        Some(chat) => {
            let user_name = {
                let clients_w = clients.write().await;
                let user = clients_w.get(user_id).unwrap();
                user.user_name.clone()
            };
            broadcast_msg(ServerWsMessage::SystemMessage(format!("{} joined chat", user_name)), chat, clients).await;
            // write to clients AFTER broadcast/inside scope
            let mut clients_w = clients.write().await;
            let user = clients_w.get_mut(user_id).unwrap();
            user.chat = Some(chat.chat_id.clone());
            send_msg(user, ServerWsMessage::SystemEvent(ServerEvent::ChatAccept(chat.chat_id.clone(), chat_title))).await;
        }
    }
}

pub(crate) async fn leave_chat(user_id: &UserId, clients: &Clients, chats: &Chats){
    let chat_id = {
        let cid = clients.write().await.get(user_id).unwrap().chat.clone();
        if cid == None {
            return;
        }
        cid.unwrap()
    };
    let mut chats_w = chats.write().await;
    if {
        let chat = chats_w.get_mut(&chat_id).unwrap();
        debug!("{} left chat {}", user_id, chat_id);
        let name = {
            let mut c = clients.write().await;
            let mut user = c.get_mut(user_id).unwrap();
            user.chat = None;
            user.user_name.clone()
        };
        broadcast_msg(ServerWsMessage::SystemMessage(format!("{} left chat", name)), chat, clients).await;
        if user_id == &chat.owner {
            debug!("disbanded chat {}", chat_id);
            broadcast_msg(ServerWsMessage::SystemMessage(format!("{} disbanded chat", name)), chat, clients).await;
            for user in chat.users.iter() {
                if user_id == &chat.owner {
                    continue
                }
                for u_name in clients.write().await.get_mut(user).map(|c| {
                    c.chat = None;
                    debug!("{} left chat {}", user, chat_id);
                    &c.user_name
                }) {
                    broadcast_msg(ServerWsMessage::SystemMessage(format!("{} left chat", u_name)), chat, clients).await;
                }
            }
        }
        chat.users.remove(user_id);
        user_id == &chat.owner
    } {
        chats_w.remove(&chat_id);
    }
}

pub(crate) async fn broadcast_msg(message: ServerWsMessage, chat: &Chat, clients: &Clients){
    let c = clients.read().await;
    for user in chat.users.iter() {
        send_msg(c.get(user).unwrap(), message.clone()).await;
    }
}

pub(crate) async fn send_msg(client: &Client, message: ServerWsMessage) {
    if let Some(sender) = &client.sender {
        let _ = sender.send(Ok(Message::text(serialize(&message).unwrap())));
    }
}