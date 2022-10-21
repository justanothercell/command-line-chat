use crate::{ws, Client, Clients, Result, debug, Chats, SERVER_VERSION};
use clc_lib::protocol::{Response, ServerConnectRequest, ServerConnectResponse, ServerDisconnectRequest, ServerDisconnectResponse, UserId, UserName};
use serde::{Deserialize};
use uuid::Uuid;
use warp::{reply::json, Reply};
use clc_lib::validator::is_valid_name;
use crate::chat::leave_chat;

#[derive(Deserialize, Debug)]
pub(crate) struct Event {
    topic: String,
    user_id: Option<usize>,
    message: String,
}

pub(crate) async fn register(body: ServerConnectRequest, clients: Clients) -> Result<impl Reply> {
    let name = body.0.trim().to_string() as UserName;

    if name.len() < 3 || name.len() > 16 {
        return Ok(json(&Response::<ServerConnectResponse>::Fail(format!("name should be between 3 and 16 characters long, found {}", name.len()))))
    }

    if !is_valid_name(&name) {
        return Ok(json(&Response::<ServerConnectResponse>::Fail(format!("name is not valid"))))
    }

    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), name, clients).await;
    debug!("{} registered", uuid);
    Ok(json(&Response::Accept(ServerConnectResponse(uuid, SERVER_VERSION.to_string()))))
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
        if let Client { chat: Some(_), .. } = client.clone() {
            leave_chat(&request.0, &clients, &chats).await;
        }
        Ok(json(&Response::Accept(ServerDisconnectResponse())))
    }
    else{
        Ok(json(&Response::<ServerDisconnectResponse>::Fail(format!("Invalid user id"))))
    }
}

pub(crate) async fn ws_handler(ws: warp::ws::Ws, id: UserId, clients: Clients, chats: Chats) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => {
            debug!("Created websocket connection for {}", c.user_id);
            Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, chats)))
        }
        None => Err(warp::reject::not_found()),
    }
}