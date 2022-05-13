use actix::prelude::*;
use uuid::Uuid;

/// New chat session is created
#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient<Message>,
}

/// Join to room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: Uuid,
    pub name: String,
    pub room: String,
}

/// Leave room
#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub id: Uuid,
}

/// Room Message
#[derive(Message)]
#[rtype(result = "()")]
pub struct RoomMessage {
    pub id: Uuid,
    pub name: String,
    pub room: String,
    pub msg: String,
}

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
}

/// Chat server sends this messages to session
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Message(pub String);
