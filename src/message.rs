use actix::prelude::*;
use std::collections::HashSet;

/// Chat server sends this messages to session
#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub json::JsonValue);

/// New chat session is created
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub name: String,
    pub addr: Recipient<Message>,
}

/// Join to room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: String,
    pub name: String,
    pub room: String,
}

/// Leave room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub id: String,
    pub room: String,
}

/// Room
#[derive(Debug)]
pub struct Room {
    pub name: String,
    pub sockets: HashSet<String>,
}

/// Room Message
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub id: String,
    pub name: String,
    pub room: String,
    pub message: String,
}

/// Private Message
#[derive(Message)]
#[rtype(result = "()")]
pub struct PrivateMessage {
    pub emitter: String,
    pub receiver: String,
    pub message: String,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    pub room: Option<String>,
}

/// Profile
#[derive(Message)]
#[rtype(result = "()")]
pub struct Profile {
    pub id: String,
    pub name: String,
}

/// Socket
#[derive(Debug)]
pub struct Socket {
    pub name: String,
    pub addr: Recipient<Message>,
}
