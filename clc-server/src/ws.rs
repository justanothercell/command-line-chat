use crate::{Client, Clients, debug};
use futures::{FutureExt, StreamExt};
use serde::Deserialize;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};
use clc_lib::deserialize;
use clc_lib::protocol::{ClientWsMessage, UserId};

pub(crate) async fn client_connection(ws: WebSocket, user_id: UserId, clients: Clients) {
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
        client_msg(&user_id, msg, &clients).await;
    }

    clients.write().await.remove(&user_id);
    debug!("{} disconnected + unregistered", user_id);
}

async fn client_msg(client_id: &UserId, msg: Message, clients: &Clients) {
    debug!("received message from {}: {:?}", client_id, msg);
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
            if let Some(chat_id) = clients.read().await.get(client_id) {

            }
        }
    }
}