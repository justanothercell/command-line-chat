use crate::{Chats, Clients, debug};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use clc_lib::deserialize;
use clc_lib::protocol::{ClientWsMessage, ServerWsMessage, UserId};
use crate::chat::{broadcast_msg, create_chat, create_chat_invite, join_chat, leave_chat, send_msg};

pub(crate) async fn client_connection(ws: WebSocket, user_id: UserId, clients: Clients, chats: Chats) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));
    clients.write().await.get_mut(&user_id).unwrap().sender = Some(client_sender);

    debug!("{} connected", user_id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", user_id, e);
                break;
            }
        };
        client_msg(&user_id, msg, &clients, &chats).await;
    }
    leave_chat(&user_id, &clients, &chats).await;
    clients.write().await.remove(&user_id);
    debug!("{} disconnected + unregistered", user_id);
}

async fn client_msg(client_id: &UserId, msg: Message, clients: &Clients, chats: &Chats) {
    debug!("received ws message from {}: {:?}", client_id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message.trim() == "ping" {
        return;
    }

    let cwsm: ClientWsMessage = match deserialize(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to topics request: {}", e);
            return;
        }
    };

    match cwsm {
        ClientWsMessage::Message(content) => {
            let (name, chat) = {
                let clients_r = clients.read().await;
                let c = clients_r.get(client_id).unwrap();
                (c.user_name.clone(), c.chat.clone())
            };
            if let Some(chat_id) = chat {
                broadcast_msg(ServerWsMessage::Message(client_id.to_string(), name, content), chats.read().await.get(&chat_id).unwrap(), clients).await;
            }
        }
        ClientWsMessage::ChatCreate(title) => {
            create_chat(title, client_id, clients, chats).await;
        }
        ClientWsMessage::ChatCreateInvite => {
            create_chat_invite(client_id, clients, chats).await;
        }
        ClientWsMessage::ChatJoin(chat_title, invite_id) => {
            join_chat(client_id, chat_title, invite_id, clients, chats).await;
        }
        ClientWsMessage::ChatLeave => {
            leave_chat(client_id, clients, chats).await;
        }
        ClientWsMessage::ChatListMembers => {
            let clients_r = clients.read().await;
            let c = clients_r.get(client_id).unwrap();
            if let Some(chat_id) = c.chat.as_ref() {
                let chats_r = chats.read().await;
                let chat = chats_r.get(chat_id).unwrap();
                let mut response = String::new();
                response.push_str(&format!("members of {}:\n", chat.title));
                for user in chat.users.iter() {
                    response.push_str(&format!("    {}\n", clients_r.get(user).unwrap().user_name));
                }
                response.push_str(&format!("open invites:\n"));
                for invite in chat.invites.iter() {
                    response.push_str(&format!("    {}\n", invite));
                }
                send_msg(c, ServerWsMessage::SystemMessage(response)).await;
            }
        }
    }
}