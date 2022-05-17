use actix::prelude::*;

/// New chat session is created
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: String,
    pub addr: Recipient<Message>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Name {
    pub name: String,
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
#[derive(Hash, Eq, PartialEq, Debug, Message)]
#[rtype(result = "()")]
pub struct Room {
    pub name: String,
    pub people: usize,
    pub id: String,
    pub sockets: Vec<String>,
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

/// Session is disconnected
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: String,
    pub room: String,
}

/// Chat server sends this messages to session
#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct Message(pub String);


