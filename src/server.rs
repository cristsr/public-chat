use actix::prelude::*;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use uuid::Uuid;

use crate::config::generate_rooms;
use crate::message;
use crate::message::{Connect, Disconnect, Join, Leave, Room, RoomMessage};

#[derive(Debug)]
pub struct ChatServer {
    sockets: HashMap<String, Recipient<message::Message>>,
    rooms: HashSet<Room>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sockets: HashMap::new(),
            rooms: generate_rooms(),
        }
    }

    pub fn get_room(&self, room_id: &String) -> Option<&Room> {
        self.rooms.iter().find(|r| r.id == room_id.clone())
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) -> Self::Result {
        log::info!("New connection: {}", msg.id);

        self.sockets.insert(msg.id, msg.addr);

        return 1;
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) {
        let room = match self.get_room(&msg.room) {
            Some(room) => room,
            None => {
                log::info!("Room {} not found", msg.room);
                return;
            }
        };

        log::info!("Room {} found", room.id);

        let mut room_sockets = &room.sockets;

        room_sockets.insert(msg.id);

        log::info!("{:?} joined to room {}", msg.id, msg.room);

        room_sockets.iter().for_each(|socket| {
            if !self.sockets.contains_key(socket) {
                log::info!("Socket is not registered {} ", socket);
                return;
            }

            log::info!("Socket is registered {}", socket);

            self.sockets
                .get_mut(socket)
                .unwrap()
                .do_send(message::Message(
                    json!({
                        "event": "userConnected",
                        "data": {
                            "id": msg.id,
                            "name": msg.name,
                        },
                    })
                    .to_string(),
                ));
        });
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, ctx: &mut Self::Context) {
        // if !self.rooms.contains_key(&msg.room) {
        //     return;
        // }

        // self.rooms.get_mut(&msg.room);

        // return ();
    }
}

impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Self::Context) {
        let room = match self.get_room(&msg.room) {
            Some(room) => room,
            None => {
                log::info!("Room {} not found", msg.room);
                return;
            }
        };

        log::info!("Room {} found", room.id);

        room.sockets.iter().for_each(|socket| {
            if !self.sockets.contains_key(socket) {
                log::info!("Socket is not registered {} ", socket);
                return;
            }

            log::info!("Socket is registered {}", socket);

            self.sockets
                .get_mut(socket)
                .unwrap()
                .do_send(message::Message(
                    json!({
                        "event": "message",
                        "data": {
                            "id": msg.id,
                            "name": msg.name,
                            "message": msg.message,
                            "room": msg.room,
                        },
                    })
                    .to_string(),
                ));
        });
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {}
}
