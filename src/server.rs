use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

use crate::message::{
    Connect, Disconnect, Join, Leave, Message, PrivateMessage, Profile, Room, RoomMessage, Socket,
};

#[derive(Debug)]
pub struct ChatServer {
    sockets: HashMap<String, Socket>,
    rooms: HashMap<String, Room>,
}

impl ChatServer {
    pub fn new() -> ChatServer {
        let mut rooms = HashMap::new();

        // Add default rooms
        ["Amistad", "Porno", "Maduritas", "Colombia", "Latinos"]
            .iter()
            .for_each(|name| {
                rooms.insert(
                    Uuid::new_v4().to_string(),
                    Room {
                        name: String::from(name.clone()),
                        sockets: HashSet::new(),
                    },
                );
            });

        ChatServer {
            sockets: HashMap::new(),
            rooms,
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        log::info!("New connection: {}", msg.id);

        // Notify available rooms to socket
        msg.addr.do_send(Message(object! {
            event: "rooms",
            data: self
                .rooms
                .iter()
                .map(|(key, value)| {
                    object! {
                        id: key.clone(),
                        name: value.name.clone(),
                        people: value.sockets.len()
                    }
                })
                .collect::<Vec<json::JsonValue>>(),
        }));

        self.sockets.insert(
            msg.id,
            Socket {
                name: msg.name,
                addr: msg.addr,
            },
        );
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, ctx: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::error!("Room not found {}", msg.room);
            return;
        }

        if !self.sockets.contains_key(&msg.id) {
            log::error!("Socket not found {}", msg.id);
            return;
        }

        // Get room
        let room = self.rooms.get_mut(&msg.room).unwrap();

        // Notify room members
        let users = room
            .sockets
            .iter()
            .filter(|id| self.sockets.contains_key(*id))
            .map(|id| (self.sockets.get(id).unwrap(), id))
            .map(|(socket, id)| {
                socket.addr.do_send(Message(object! {
                    event: "userConnected",
                    data: {
                        id: msg.id.clone(),
                        name: msg.name.clone(),
                        room: msg.room.clone(),
                    },
                }));

                object! {
                    id: id.clone(),
                    name: socket.name.clone(),
                }
            })
            .collect::<Vec<json::JsonValue>>();

        // Add client to room
        room.sockets.insert(msg.id.clone());

        // Notify client users in room
        self.sockets.get(&msg.id).unwrap().addr.do_send(Message(
            object! {
                event: "usersInRoom",
                data: {
                    room: msg.room.clone(),
                    users: users,
                },
            }
        ));

        log::info!("Socket {} joined room {}", msg.id, msg.room);
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::error!("Room {} not found", msg.room);
            return;
        }

        // Get room
        let room = self.rooms.get_mut(&msg.room).unwrap();

        // Remove socket from room
        room.sockets.retain(|socket| socket != &msg.id);

        log::info!("{} left room {}", &msg.id, &msg.room);

        // Notify room about user disconnect
        room.sockets
            .iter()
            .filter(|id| self.sockets.contains_key(*id))
            .map(|id| self.sockets.get(id).unwrap())
            .for_each(|socket| {
                socket.addr.do_send(Message(object! {
                    event: "leaveRoom",
                    data: {
                        id: msg.id.clone(),
                        room: msg.room.clone(),
                    }
                }));
            });
    }
}

impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, _: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::error!("Room {} not found", msg.room);
            return;
        }

        // Get room
        let room = self.rooms.get(&msg.room).unwrap();

        // Notify room members
        room.sockets.iter().for_each(|socket| {
            if !self.sockets.contains_key(socket) {
                log::error!("Socket {} not found", socket);
                return;
            }

            self.sockets
                .get(socket)
                .unwrap()
                .addr
                .do_send(Message(object! {
                    event: "roomMessage",
                    data: {
                        id: msg.id.clone(),
                        name: msg.name.clone(),
                        message: msg.message.clone(),
                        room: msg.room.clone(),
                    },
                }));
        });

        log::info!("Message sent to room {}", msg.room);
    }
}

impl Handler<PrivateMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: PrivateMessage, _: &mut Self::Context) {
        // Verify if emmiter is connected
        if !self.sockets.contains_key(&msg.emitter) {
            log::error!("Socket {} not found", msg.emitter);
            return;
        }

        // Verify if receiver is connected
        if !self.sockets.contains_key(&msg.receiver) {
            log::error!("Socket {} not found", msg.receiver);
            return;
        }

        let payload = object! {
            event: "privateMessage",
            data: {
                emmiter: msg.emitter.clone(),
                receiver: msg.receiver.clone(),
                message: msg.message.clone(),
            },
        };

        self.sockets
            .get(&msg.emitter)
            .unwrap()
            .addr
            .do_send(Message(payload.clone()));

        self.sockets
            .get(&msg.receiver)
            .unwrap()
            .addr
            .do_send(Message(payload.clone()));

        log::info!(
            "Private message sent from {} to {}",
            msg.emitter,
            msg.receiver
        );
    }
}

impl Handler<Profile> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Profile, _: &mut Self::Context) -> Self::Result {
        // Verify if socket is connected
        if !self.sockets.contains_key(&msg.id) {
            log::error!("Socket {} not found", msg.id);
            return;
        }

        // Get socket
        let socket = self.sockets.get_mut(&msg.id).unwrap();

        // Update socket name
        socket.name = msg.name;

        // Notify user profile
        socket.addr.do_send(Message(object! {
            event: "profile",
            data: {
                id: msg.id.clone(),
                name: socket.name.clone(),
            },
        }));
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Self::Context) -> Self::Result {
        log::info!("Socket disconnect {}", msg.id);

        // Verify if room is defined
        if let Some(r) = msg.room {
            // Get room
            let room = self.rooms.get_mut(&r).unwrap();

            // Remove socket from room
            room.sockets.remove(&msg.id);

            // Notify room members
            room.sockets.iter().for_each(|socket| {
                if !self.sockets.contains_key(socket) {
                    log::error!("Socket {} not found", socket);
                    return;
                }

                self.sockets
                    .get(socket)
                    .unwrap()
                    .addr
                    .do_send(Message(object! {
                        event: "leaveRoom",
                        data: {
                            id: msg.id.clone(),
                            room: r.clone(),
                        }
                    }));
            });
        }

        // Remove socket from server
        self.sockets.remove(&msg.id);
    }
}
