use serde::{Serialize, Deserialize};

pub type UserName = String;
pub type UserId = String;
pub type ChatTitle = String;
pub type ChatId = String;
pub type InviteId = String;
pub type ServerUrl = String;
pub type FilePath = String;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectRequest(pub UserName);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConnectResponse(pub UserId);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectRequest(pub UserId);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerDisconnectResponse();