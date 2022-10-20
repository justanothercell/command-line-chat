use std::fmt::{Display, Formatter};
use reqwest::{Url};
use reqwest::blocking::RequestBuilder;
use serde::{Deserialize, Serialize};
use clc_lib::{deserialize, serialize};
use clc_lib::protocol::{ServerConnectRequest, ServerConnectResponse, ServerDisconnectRequest, ServerDisconnectResponse, ServerUrl, UserName};
use crate::Client;
use crate::client::{ClientSeal, ThreadClient};

enum Method {
    Get,
    Post,
    Delete
}

impl Client {
    pub(crate) fn connect_server(client: &ThreadClient, url: ServerUrl, name: UserName) {
        match Self::request(Method::Post, format!("http://{}/register", url), &ServerConnectRequest(name.clone())) {
            Ok(ServerConnectResponse(uuid)) => {
                let mut c = client.seal();
                c.server = Some(url.clone());
                c.user_id = Some(uuid);
                c.loc = Location::Lobby;
                c.writeln(&format!("Connected to server {} as {}", url, name));
            }
            Err(e) => {
                client.seal().writeln(&format!("Unable to connect to server {} as {}: {}", url, name, e));
            }
        }
    }

    pub(crate) fn disconnect_server(client: &ThreadClient) {
        let url = client.seal().server.as_ref().unwrap().clone();
        let user_id = client.seal().user_id.as_ref().unwrap().clone();
        match Self::request(Method::Delete, format!("http://{}/register/{}", url, user_id), &ServerDisconnectRequest(user_id)) {
            Ok(ServerDisconnectResponse()) => {
                let mut c = client.seal();
                c.server = None;
                c.user_id = None;
                c.loc = Location::Home;
                c.writeln(&format!("Disconnected from server {}", url));
            }
            Err(e) => {
                client.seal().writeln(&format!("Unable to disconnect from server {}: {}", url, e));
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