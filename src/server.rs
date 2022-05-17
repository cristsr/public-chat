use actix::prelude::*;
use serde_json::json;
use std::collections::HashMap;

use crate::config::generate_rooms;
use crate::message;
use crate::message::{Connect, Disconnect, Join, Leave, Room, RoomMessage};

#[derive(Debug)]
pub struct ChatServer {
    sockets: HashMap<String, Recipient<message::Message>>,
    rooms: Vec<Room>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        ChatServer {
            sockets: HashMap::new(),
            rooms: generate_rooms(),
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, ctx: &mut Self::Context) {
        log::info!("New connection: {}", msg.id);

        let rooms = self
            .rooms
            .iter()
            .map(|room| {
                json!({
                    "name": room.name,
                    "id": room.id,
                    "people": room.people,
                })
                .to_string()
            })
            .collect::<Vec<String>>();

        msg.addr.do_send(message::Message(
            json!({
                "event": "rooms",
                "data": rooms,
            })
            .to_string(),
        ));

        self.sockets.insert(msg.id, msg.addr);
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Self::Context) {
        let room: &mut Room = match self.rooms.iter_mut().find(|r| r.id == msg.room) {
            Some(room) => room,
            None => {
                log::info!("Room {} not found", &msg.room);
                return;
            }
        };

        log::info!("Room {} found", &room.id);

        if !room.sockets.contains(&msg.id) {
            // Add socket to room
            room.sockets.push(msg.id.clone());
        }

      

        log::info!("{:?} joined to room {}", &msg.id, &msg.room);

        room.sockets.iter().for_each(|socket| {
            log::info!("Sending to {}", socket);

            if !self.sockets.contains_key(socket) {
                log::info!("Socket is not registered {} ", socket);
                return;
            }

            log::info!("Socket is registered {}", socket);

            // Notify room about new user
            self.sockets
                .get_mut(socket)
                .unwrap()
                .do_send(message::Message(
                    json!({
                        "event": "userConnected",
                        "data": {
                            "id": msg.id,
                            "name": msg.name,
                            "room": msg.room,
                        }
                    })
                    .to_string(),
                ));
        });
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) {
        let room: &mut Room = match self.rooms.iter_mut().find(|r| r.id == msg.room) {
            Some(room) => room,
            None => {
                log::info!("Room {} not found", &msg.room);
                return;
            }
        };

        // Remove socket from room
        room.sockets.retain(|socket| socket != &msg.id);

        log::info!("{} left room {}", &msg.id, &msg.room);

        room.sockets.iter().for_each(|socket| {
            log::info!("Sending to {}", socket);

            if !self.sockets.contains_key(socket) {
                log::info!("Socket is not registered {} ", socket);
                return;
            }

            log::info!("Socket is registered {}", socket);

            // Notify room about user disconnect
            self.sockets
                .get_mut(socket)
                .unwrap()
                .do_send(message::Message(
                    json!({
                        "event": "leaveRoom",
                        "data": {
                            "id": msg.id,
                            "room": msg.room,
                        }
                    })
                    .to_string(),
                ));
        });
    }
}

impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, _: &mut Self::Context) {
        let room: &mut Room = match self.rooms.iter_mut().find(|r| r.id == msg.room) {
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

            log::info!("Socket is registered {}", &socket);

            // Send message to all sockets in room
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

    fn handle(&mut self, msg: Disconnect, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Socket disconnect {}", msg.id);

        match self.rooms.iter_mut().find(|r| r.id == msg.room) {
            Some(room) => {
                // Remove socket from room
                room.sockets.retain(|socket| socket != &msg.id);

                // Notify room user disconnected
                room.sockets.iter().for_each(|socket| {
                    log::info!("Sending to {}", socket);

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
                                 "event": "leaveRoom",
                                 "data": {
                                      "id": msg.id,
                                      "room": room.id,
                                 }
                            })
                            .to_string(),
                        ));
                });
            }
            None => {
                log::info!("Room not found: {}", msg.id);
            }
        }
    }
}
