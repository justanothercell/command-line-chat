use std::net::TcpStream;
use std::sync::mpsc::channel;
use std::thread;
use native_tls::{TlsConnector, TlsConnectorBuilder, TlsStream};
use reqwest::Url;
use tungstenite::{accept, connect, Message};
use tungstenite::client::client as create_client;
use tungstenite::stream::MaybeTlsStream;
use clc_lib::deserialize;
use clc_lib::protocol::{ServerEvent, ServerWsMessage};
use crate::client::{ClientSeal, ThreadClient};
use crate::web_client::Location;

pub(crate) fn create_ws_connection(client: &ThreadClient){
    let url = {
        let c = client.seal();
        &format!("wss://{}/ws/{}", c.server.as_ref().unwrap(), c.user_id.as_ref().unwrap())
    };
    println!("OOOOOO");
    let (mut socket, _response) = connect(url).expect("Can't connect");
    match socket.get_mut() {
        MaybeTlsStream::NativeTls(stream) => {
            stream.get_mut().set_nonblocking(true).expect("Unable to set nonblocking");
        }
        _ => unreachable!()
    }
    println!("222222");
    let ws_client = client.clone();
    let (tx, rx) = channel();
    let socket_thread = thread::spawn(move || {
        loop {
            // === receive message from client and send to server ===
            let message = match rx.recv() {
                Ok(Message::Close(f)) => {
                    ws_client.seal().writeln(&format!("Websocket closed"));
                    let _ = socket.write_message(Message::Close(f));
                    return;
                }
                Ok(m) => m,
                Err(e) => {
                    ws_client.seal().writeln(&format!("Websocket thread error: {}", e));
                    return;
                }
            };
            match socket.write_message(message) {
                Ok(()) => (),
                Err(e) => {
                    ws_client.seal().writeln(&format!("Websocket send error: {:?}", e));
                    let _ = socket.write_message(Message::Close(None));
                    return;
                }
            }

            // === receive message from server ===
            match socket.read_message() {
                Err(_) => {} // no message on nonblocking
                Ok(message) => {
                    match message {
                        Message::Text(content) => {
                            let msg = deserialize(&content).unwrap();
                            receive_ws_message(msg, &ws_client);
                        }
                        Message::Binary(_) => panic!("Unsupported"),
                        Message::Ping(data) => { let _ = socket.write_message(Message::Pong(data)); },
                        Message::Pong(_) => {/* ponged */}
                        Message::Close(_) => {
                            let _ = socket.write_message(message);
                            ws_client.seal().writeln(&format!("Websocket send_thread closed"));
                            return;
                        }
                        Message::Frame(_) => unreachable!("Docs say this is unobtainable with reading")
                    }
                }
            }
        }
    });
    {
        let mut c = client.seal();
        c.socket = Some(socket_thread);
        c.sender = Some(tx);
        c.writeln("Created websocket connection");
    }
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