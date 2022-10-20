use futures::SinkExt;
use crate::{ws, Client, Clients, Result, debug, Chats, Chat};
use clc_lib::protocol::{ChatCreateRequest, ChatCreateResponse, ChatId, Response, ServerConnectRequest, ServerConnectResponse, ServerDisconnectRequest, ServerDisconnectResponse, UserId, UserName};
use serde::{Deserialize};
use uuid::Uuid;
use warp::{http::StatusCode, reply::json, ws::Message, Reply};

#[derive(Deserialize, Debug)]
pub(crate) struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

pub(crate) async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    clients
        .read()
        .await
        .iter()
        .filter(|(_, client)| match body.user_id {
            Some(v) => &client.user_name == "",
            None => true,
        })
        .filter(|(_, client)| client.chat.as_ref().map_or(false, |c| c== "0"))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        });

    Ok(StatusCode::OK)
}

pub(crate) async fn register(body: ServerConnectRequest, clients: Clients) -> Result<impl Reply> {
    let name = body.0.trim().to_string() as UserName;

    if name.len() < 3 || name.len() > 16 {
        return Ok(json(&Response::<ServerConnectResponse>::Fail(format!("name should be between 3 and 16 characters long, found {}", name.len()))))
    }

    if let Err(_) = syn::parse_str::<syn::Ident>(&name) {
        return Ok(json(&Response::<ServerConnectResponse>::Fail(format!("name should be a valid identifier"))))
    }

    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), name, clients).await;
    debug!("{} registered", uuid);
    Ok(json(&Response::Accept(ServerConnectResponse(uuid))))
}

async fn register_client(user_id: UserId, name: UserName, clients: Clients) {
    clients.write().await.insert(
        user_id.clone(),
        Client {
            user_id,
            user_name: name,
            chat: None,
            sender: None,
        },
    );
}

pub(crate) async fn unregister(request: ServerDisconnectRequest, clients: Clients, chats: Chats) -> Result<impl Reply> {
    if let Some(client) = clients.write().await.remove(&request.0){
        debug!("{} unregistered", request.0);
        if let Client { chat: Some(chat_id), .. } = client.clone() {
            leave_chat(&clients, &chats, &request.0, &chat_id).await;
        }
        Ok(json(&Response::Accept(ServerDisconnectResponse())))
    }
    else{
        Ok(json(&Response::<ServerDisconnectResponse>::Fail(format!("Invalid user id"))))
    }
}

pub(crate) async fn create_chat(request: ChatCreateRequest, clients: Clients, chats: Chats) -> Result<impl Reply> {
    let name = request.1;
    if name.len() < 3 || name.len() > 24 {
        return Ok(json(&Response::<ChatCreateResponse>::Fail(format!("title should be between 3 and 24 characters long, found {}", name.len()))))
    }

    if let Err(_) = syn::parse_str::<syn::Ident>(&name) {
        return Ok(json(&Response::<ChatCreateResponse>::Fail(format!("title should be a valid identifier"))))
    }
    
    let mut chats_w = chats.write().await;
    Ok(clients.write().await.get_mut(&request.0)
        .map_or(json(&Response::<ChatCreateResponse>::Fail(format!("Invalid user id"))), |c| {
            let uuid = Uuid::new_v4().as_simple().to_string();
            debug!("{} created chat {}", request.0, uuid);
            c.chat = Some(uuid.clone());
            chats_w.insert(
                uuid.clone(),
                Chat {
                    chat_id: uuid.clone(),
                    title: name,
                    owner: request.0,
                    users: Default::default(),
                    invites: Default::default()
                },
            );
            json(&Response::Accept(ChatCreateResponse(uuid)))
        }))
}

async fn leave_chat(clients: &Clients, chats: &Chats, user_id: &UserId, chat_id: &ChatId){
    let mut chats_w = chats.write().await;
    if let Some(chat) = chats_w.get_mut(chat_id){
        chat.users.remove(user_id);
        debug!("{} left chat {}", user_id, chat_id);
        if user_id == &chat.owner {
            debug!("disbanded chat {}", chat_id);
            let mut clients_w = clients.write().await;
            for user in chat.users.iter() {
                clients_w.get_mut(user).map(|c| {
                    c.chat = None;
                    debug!("{} left chat {}", user, chat_id);
                });
            }
            chats_w.remove(chat_id);
        }
    }
}

pub(crate) async fn ws_handler(ws: warp::ws::Ws, id: UserId, clients: Clients) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => {
            debug!("Created websocket connection for {}", c.user_id);
            Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients)))
        }
        None => Err(warp::reject::not_found()),
    }
}