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
        ["Amistad", "Videojuegos", "Anime", "Colombia", "Latinoamerica"]
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

    fn notify_people_in_rooms(&self) {
        let rooms = self
            .rooms
            .iter()
            .map(|(key, value)| {
                object! {
                    id: key.clone(),
                    name: value.name.clone(),
                    people: value.sockets.len()
                }
            })
            .collect::<Vec<json::JsonValue>>();

        self.sockets.iter().for_each(|(_id, socket)| {
            socket.addr.do_send(Message(object! {
                event: "rooms",
                data: rooms.clone(),
            }));
        });
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Self::Context) {
        log::info!("New connection: {}", &msg.id);

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
            msg.id.clone(),
            Socket {
                name: msg.name,
                addr: msg.addr,
            },
        );
    }
}

impl Handler<Join> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Join, _: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::warn!("Room not found {}", msg.room);
            return;
        }

        // Verify if socket is connected
        if !self.sockets.contains_key(&msg.id) {
            log::warn!("Socket not found {}", msg.id);
            return;
        }

        // Verify if socket is already in room
        if self.rooms.get(&msg.room).unwrap().sockets.contains(&msg.id) {
            log::warn!("Socket already in room {}", msg.room);
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

        // Get socket joined
        let socket = self.sockets.get(&msg.id).unwrap();

        // Notify client users in room
        socket.addr.do_send(Message(object! {
            event: "usersInRoom",
            data: {
                room: msg.room.clone(),
                users: users,
            },
        }));

        // Notify to all sockets
        self.notify_people_in_rooms();

        // Notify Room members
        log::info!("Socket {} joined room {}", msg.id, msg.room);
    }
}

impl Handler<Leave> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Leave, _: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::warn!("Room not found {}", msg.room);
            return;
        }

        // Get room
        let room = self.rooms.get_mut(&msg.room).unwrap();

        // Remove socket from room
        room.sockets.remove(&msg.id);

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

        // Notify to all sockets
        self.notify_people_in_rooms();

        log::info!("{} left room {}", &msg.id, &msg.room);
    }
}

impl Handler<RoomMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, _: &mut Self::Context) {
        // Verify if room exists
        if !self.rooms.contains_key(&msg.room) {
            log::warn!("Room not found {}", msg.room);
            return;
        }

        // Get room
        let room = self.rooms.get(&msg.room).unwrap();

        // Notify room members
        room.sockets
            .iter()
            .filter(|id| self.sockets.contains_key(*id))
            .map(|id| self.sockets.get(id).unwrap())
            .for_each(|socket| {
                socket.addr.do_send(Message(object! {
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
            log::warn!("Socket not found {}", msg.emitter);
            return;
        }

        // Verify if receiver is connected
        if !self.sockets.contains_key(&msg.receiver) {
            log::warn!("Socket not found {}", msg.receiver);
            return;
        }

        let emmiter = self.sockets.get(&msg.emitter).unwrap();
        let receiver = self.sockets.get(&msg.receiver).unwrap();

        let payload = object! {
            event: "privateMessage",
            data: {
                emmiter: {
                    id: msg.emitter.clone(),
                    name: emmiter.name.clone(),
                },
                receiver: {
                    id: msg.receiver.clone(),
                    name: receiver.name.clone(),
                },
                message: msg.message.clone(),
            },
        };

        emmiter.addr.do_send(Message(payload.clone()));
        receiver.addr.do_send(Message(payload.clone()));

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
            log::warn!("Socket {} not found", msg.id);
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

        // Remove socket from server
        self.sockets.remove(&msg.id);
    }
}
