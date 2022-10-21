use std::fmt::Debug;
use serde::{Serialize, Deserialize};

pub type UserName = String;
pub type UserId = String;
pub type ChatTitle = String;
pub type ChatId = String;
pub type InviteId = String;
pub type ServerUrl = String;
pub type FilePath = String;
pub type Version = String;
pub type Reason = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Response<T> {
    Accept(T),
    Fail(Reason)
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectRequest(pub UserName);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectResponse(pub UserId, pub Version);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectRequest(pub UserId);
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectResponse();

// no request data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerVersion(pub Version);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientWsMessage{
    Message(String),
    ChatCreate(ChatTitle),
    ChatJoin(ChatId, InviteId),
    ChatLeave,
    ChatCreateInvite,
    ChatListMembers
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerWsMessage{
    Message(UserId, UserName, String),
    SystemMessage(String),
    SystemEvent(ServerEvent)
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerEvent{
    ChatCreate(ChatId, ChatTitle),
    ChatAccept(ChatId, ChatTitle),
    SetAdmin(bool)
}