use crate::{ws, Client, Clients, Result, debug};
use clc_lib::protocol::{ServerConnectRequest, ServerConnectResponse, ServerDisconnectResponse, UserId, UserName};
use serde::{Deserialize, Serialize};
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
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _ = sender.send(Ok(Message::text(body.message.clone())));
            }
        });

    Ok(StatusCode::OK)
}

pub(crate) async fn register(body: ServerConnectRequest, clients: Clients) -> Result<impl Reply> {
    let name = body.0;
    let uuid = Uuid::new_v4().as_simple().to_string();

    register_client(uuid.clone(), name, clients).await;
    debug!("{} registered", uuid);
    Ok(json(&ServerConnectResponse(uuid)))
}

async fn register_client(user_id: UserId, name: UserName, clients: Clients) {
    clients.write().await.insert(
        user_id,
        Client {
            user_name: name,
            topics: vec![String::from("cats")],
            sender: None,
        },
    );
}

pub(crate) async fn unregister(id: String, clients: Clients) -> Result<impl Reply> {
    debug!("{} unregistered", id);
    clients.write().await.remove(&id);
    Ok(json(&ServerDisconnectResponse()))
}

pub(crate) async fn ws_handler(ws: warp::ws::Ws, id: String, clients: Clients) -> Result<impl Reply> {
    let client = clients.read().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
}

pub(crate) async fn health() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}