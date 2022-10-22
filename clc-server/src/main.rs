use std::collections::{HashMap, HashSet};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};
use warp::http::StatusCode;
use clc_lib::protocol::{ChatId, ChatTitle, InviteId, ServerVersion, UserId, UserName};

mod handler;
mod ws;
mod chat;

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    () => {
        println!()
    };
    ($($arg:tt)*) => {
        println!($($arg)*)
    };
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    () => {};
    ($($arg:tt)*) => {};
}

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<UserId, Client>>>;
type Chats = Arc<RwLock<HashMap<ChatId, Chat>>>;

const SERVER_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone)]
pub(crate) struct Client {
    pub(crate) user_id: UserId,
    pub(crate) user_name: UserName,
    pub(crate) chat: Option<ChatId>,
    pub(crate) sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Debug, Clone)]
pub(crate) struct Chat {
    pub(crate) chat_id: ChatId,
    pub(crate) title: ChatTitle,
    pub(crate) owner: UserId,
    pub(crate) users: HashSet<UserId>,
    pub(crate) invites: HashSet<InviteId>,
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let chats: Chats = Arc::new(RwLock::new(HashMap::new()));

    // auto-loads https://github.com/DragonFIghter603/command-line-chat/blob/master/index.html
    let index_route = warp::path!().and_then(|| async {
        Ok::<_, Rejection>(warp::reply::html(include_str!("../index_loader.html")))
    });

    let health_route = warp::path!("api"/"health").and_then(|| async { Ok::<_, Rejection>(StatusCode::OK) });
    let version_route = warp::path!("api"/"version")
        .and_then(|| async { Ok::<_, Rejection>(warp::reply::json(&ServerVersion(SERVER_VERSION.to_string()))) });

    let register = warp::path!("api"/"register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with(clients.clone()))
        .and_then(handler::register)
        .or(register
            .and(warp::delete())
            .and(warp::body::json())
            .and(with(clients.clone()))
            .and(with(chats.clone()))
            .and_then(handler::unregister));

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with(clients.clone()))
        .and(with(chats.clone()))
        .and_then(handler::ws_handler);

    let routes = index_route
        .or(health_route)
        .or(version_route)
        .or(register_routes)
        .or(ws_route)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes)
        //.tls()
        //.cert_path("tls/cert.pem")
        //.key_path("tls/key.rsa")

        .run(([0, 0, 0, 0], 10000)).await;
}

fn with<T: Clone + Send>(data: T) -> impl Filter<Extract = (T,), Error = Infallible> + Clone {
    warp::any().map(move || data.clone())
}