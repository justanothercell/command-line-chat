use std::sync::mpsc::channel;
use std::thread;
use websocket::{ClientBuilder, Message, OwnedMessage};
use clc_lib::deserialize;
use clc_lib::protocol::{ServerEvent, ServerWsMessage};
use crate::client::{ClientSeal, ThreadClient};
use crate::web_client::Location;

pub(crate) fn create_ws_connection(client: &ThreadClient){
    let ws_client = {
        let c = client.seal();
        ClientBuilder::new(&format!("ws://{}/ws/{}", c.server.as_ref().unwrap(), c.user_id.as_ref().unwrap()))
            .unwrap()
            .add_protocol("rust-websocket")
            .connect_insecure()
            .unwrap()
    };
    let (mut receiver, mut sender) = ws_client.split().unwrap();
    let (tx, rx) = channel();

    let tx_1 = tx.clone();
    let s_client = client.clone();
    let send_loop = thread::spawn(move || {
        loop {
            let message = match rx.recv() {
                Ok(m) => m,
                Err(e) => {
                    s_client.seal().writeln(&format!("Websocket send_thread error: {}", e));
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    let _ = sender.send_message(&message);
                    s_client.seal().writeln(&format!("Websocket send_thread closed"));
                    return;
                }
                _ => (),
            }
            match sender.send_message(&message) {
                Ok(()) => (),
                Err(e) => {
                    s_client.seal().writeln(&format!("Websocket send error: {}", e));
                    let _ = sender.send_message(&Message::close());
                    return;
                }
            }
        }
    });
    let r_client = client.clone();
    let receive_loop = thread::spawn(move || {
        for message in receiver.incoming_messages() {
            let message = match message {
                Ok(m) => m,
                Err(e) => {
                    r_client.seal().writeln(&format!("Websocket receive_thread error: {}", e));
                    let _ = tx_1.send(OwnedMessage::Close(None));
                    return;
                }
            };
            match message {
                OwnedMessage::Close(_) => {
                    let _ = tx_1.send(OwnedMessage::Close(None));
                    r_client.seal().writeln(&format!("Websocket receive_thread closed"));
                    return;
                }
                OwnedMessage::Ping(data) => {
                    match tx_1.send(OwnedMessage::Pong(data)) {
                        Ok(()) => (),
                        Err(e) => {
                            r_client.seal().writeln(&format!("Websocket receive_thread error: {}", e));
                            return;
                        }
                    }
                }
                OwnedMessage::Text(content) => {
                    let msg = deserialize(&content).unwrap();
                    receive_ws_message(msg, &r_client);
                },
                _ => r_client.seal().writeln(&format!("received unexpected ws data: {:?}", message)),
            }
        }
    });
    {
        let mut c = client.seal();
        c.socket = Some((send_loop, receive_loop));
        c.sender = Some(tx);
    }
    client.seal().writeln("Created websocket connection");
}

pub(crate) fn receive_ws_message(message: ServerWsMessage, client: &ThreadClient){
    match message {
        ServerWsMessage::Message(_sender_id, sender, content) => client.seal().writeln(&format!("[{}]: {}", sender, content)),
        ServerWsMessage::SystemMessage(content) => client.seal().writeln(&content),
        ServerWsMessage::SystemEvent(event) => match event {
            ServerEvent::ChatAccept(chat_id, chat_title) => {
                let mut c = client.seal();
                c.writeln(&format!("Joined chat {}", chat_title));
                c.chat_id = Some(chat_id);
                c.chat_title = Some(chat_title);
                c.is_admin = false;
                c.loc = Location::Chat;
            }
            ServerEvent::ChatCreate(chat_id, chat_title) => {
                let mut c = client.seal();
                c.writeln(&format!("Created chat {}", chat_title));
                c.chat_id = Some(chat_id);
                c.chat_title = Some(chat_title);
                c.is_admin = true;
                c.loc = Location::Chat;
            }
            ServerEvent::SetAdmin(is_admin) => {
                let mut c = client.seal();
                if is_admin {
                    c.writeln("You are now admin of this chat");
                }
                else {
                    c.writeln("You are no longer admin of this chat");
                }
                c.is_admin = is_admin;
            }
        }
    }
}