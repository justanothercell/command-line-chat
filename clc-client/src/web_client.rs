use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use websocket::OwnedMessage;
use clc_lib::{deserialize, serialize};
use clc_lib::protocol::{ChatCreateRequest, ChatCreateResponse, ChatTitle, ClientWsMessage, Response, ServerConnectRequest, ServerConnectResponse, ServerDisconnectRequest, ServerDisconnectResponse, ServerUrl, UserName};
use crate::Client;
use crate::client::{ClientSeal, ThreadClient};
use crate::ws_client::create_ws_connection;

enum Method {
    Get,
    Post,
    Delete
}

impl Client {
    pub(crate) fn connect_server(client: &ThreadClient, url: ServerUrl, name: UserName) {
        match Self::request(Method::Post, format!("http://{}/api/register", url), &ServerConnectRequest(name.clone())) {
            Ok(Response::Accept(ServerConnectResponse(uuid))) => {
                {
                    let mut c = client.seal();
                    c.server = Some(url.clone());
                    c.name = Some(name.clone());
                    c.user_id = Some(uuid);
                    c.loc = Location::Lobby;
                    c.writeln(&format!("Connected to server {} as {}", url, name));
                }
                create_ws_connection(client);
            }
            Ok(Response::Fail(reason)) => client.seal().writeln(&format!("Error: {}", reason)),
            Err(e) => {
                client.seal().writeln(&format!("Unable to connect to server {} as {}: {}", url, name, e));
            }
        }
    }

    pub(crate) fn disconnect_server(client: &ThreadClient) {
        let url = client.seal().server.as_ref().unwrap().clone();
        let user_id = client.seal().user_id.as_ref().unwrap().clone();
        match Self::request(Method::Delete, format!("http://{}/api/register", url), &ServerDisconnectRequest(user_id)) {
            Ok(Response::Accept(ServerDisconnectResponse())) => {
                let mut c = client.seal();
                c.server = None;
                c.user_id = None;
                c.name = None;
                c.chat_id = None;
                c.sender = None;
                let _ = c.socket.take().map(|(s, r)| {
                    s.join().expect("Unable to join send_loop thread");
                    r.join().expect("Unable to join read_loop thread");
                });
                c.loc = Location::Home;
                c.writeln(&format!("Disconnected from server {}", url));
            }
            Ok(Response::Fail(reason)) => client.seal().writeln(&format!("Error: {}", reason)),
            Err(e) => {
                client.seal().writeln(&format!("Unable to disconnect from server {}: {}", url, e));
            }
        }
    }

    pub(crate) fn send_ws_message(client: &ThreadClient, message: ClientWsMessage){
        client.seal().sender.as_ref().unwrap().send(OwnedMessage::Text(serialize(&message).expect("Unable to serialize"))).expect("Unable to send message");
    }

    pub(crate) fn create_chat(client: &ThreadClient, title: ChatTitle){
        let url = client.seal().server.as_ref().unwrap().clone();
        let user_id = client.seal().user_id.as_ref().unwrap().clone();
        match Self::request(Method::Post, format!("http://{}/api/chat", url), &ChatCreateRequest(user_id, title.clone())) {
            Ok(Response::Accept(ChatCreateResponse(chat_id))) => {
                let mut c = client.seal();
                c.chat_id = Some(chat_id.clone());
                c.loc = Location::Chat;
                c.writeln(&format!("Created chat {} with id {}", title, chat_id));
            }
            Ok(Response::Fail(reason)) => client.seal().writeln(&format!("Error: {}", reason)),
            Err(e) => {
                client.seal().writeln(&format!("Unable to create chat: {}", e));
            }
        }
    }

    fn request<B: Serialize, R: for<'a> Deserialize<'a>>(method: Method, url: String, body: &B) -> Result<R, String>{
        let client = reqwest::blocking::Client::new();
        let req = match method {
            Method::Get => client.get(url),
            Method::Post => client.post(url),
            Method::Delete => client.delete(url)
        };
        let res = req.body(serialize(body)?).send().map_err(|e| format!("{}", e))?;
        let txt = res.text().map_err(|e| format!("{}", e))?;
        deserialize(&txt)
    }
}

#[derive(Clone)]
pub(crate) enum Location {
    Home,
    Lobby,
    Chat
}

impl Display for Location {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", match self {
            Location::Home => "home",
            Location::Lobby => "lobby",
            Location::Chat => "chat"
        })
    }
}