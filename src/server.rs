use actix::prelude::*;
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::ffi::c_void;
use uuid::Uuid;

use crate::message;
use crate::message::{Connect, Disconnect, Join, RoomMessage};

#[derive(Debug)]
pub struct ChatServer {
    sockets: HashMap<Uuid, Recipient<message::Message>>,
    rooms: HashMap<String, HashSet<Uuid>>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sockets: HashMap::new(),
            rooms: HashMap::new(),
        }
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
        log::info!("{:?} joined room {}", msg.id, msg.room);

        self.rooms
            .entry(msg.room.to_owned())
            .or_insert(HashSet::new())
            .insert(msg.id);

        self.sockets.iter().for_each(|(id, addr)| {
            log::info!("{:?}", id);
        });

        log::info!("{:?}", self.sockets.len());

        self.rooms
            .get_mut(&msg.room)
            .unwrap()
            .iter()
            .for_each(|id| {
                if !self.sockets.contains_key(id) {
                    log::info!("{:?} is not connected", id);
                    return;
                }

                log::info!("{:?} sent to {}", msg.id, id);

                self.sockets.get_mut(id).unwrap().do_send(message::Message(
                    json!({
                        "event": "userConnected",
                        "data": {
                            "id": msg.id.to_string(),
                            "name": msg.name,
                        },
                    })
                    .to_string(),
                ));
            });
    }
}

impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Self::Context) -> Self::Result {
        if !self.rooms.contains_key(&msg.room) {
            log::info!("Room {} does not exist", msg.room);
            return;
        }

        self.rooms
            .get_mut(&msg.room)
            .unwrap()
            .iter()
            .for_each(|id| {
                if !self.sockets.contains_key(id) {
                    log::info!(" socket not found {:?}", id);
                    return;
                }

                self.sockets.get_mut(id).unwrap().do_send(message::Message(
                    json!({
                        "event": "roomMessage",
                        "data": {
                            "id": msg.id.to_string(),
                            "name": msg.name,
                            "msg": msg.msg,
                            "room": msg.room,
                        },
                    })
                    .to_string(),
                ));
            })
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {}
}
