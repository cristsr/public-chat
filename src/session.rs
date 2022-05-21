use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};

use crate::message::{
    Connect, Disconnect, Join, Leave, Message, PrivateMessage, Profile, RoomMessage,
};
use crate::server::ChatServer;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub id: String,
    pub name: Option<String>,
    pub room: Option<String>,
    pub hb: Instant,
    pub server: Addr<ChatServer>,
}

impl WsSession {
    /// Send ping to client every 5 seconds
    pub fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        log::info!("Start heartbeat");

        ctx.run_interval(HEARTBEAT_INTERVAL, |session, ctx| {
            if Instant::now().duration_since(session.hb) > CLIENT_TIMEOUT {
                log::info!("heartbeat: Socket {} is gone", session.id);

                // Leave room if joined to one
                if let Some(room) = &session.room {
                    session.server.do_send(Leave {
                        id: session.id.clone(),
                        room: room.clone(),
                    });
                }

                // Remove from server
                session.server.do_send(Disconnect {
                    id: session.id.clone(),
                });

                // Stop actor
                ctx.terminate();

                return;
            }

            ctx.ping(b"");
        });
    }

    fn handle_events(&mut self, event: &str, data: json::JsonValue) {
        log::info!("Event: {}", event);
        log::info!("Data: {}", data);

        match event {
            "joinRoom" => {
                if let Some(room) = &self.room {
                    self.server.do_send(Leave {
                        id: self.id.clone(),
                        room: room.clone(),
                    });
                }

                self.room = Some(data.to_string());

                self.server.do_send(Join {
                    id: self.id.clone(),
                    name: self.name.clone().unwrap_or("".to_string()),
                    room: self.room.clone().unwrap(),
                });
            }
            "leaveRoom" => {
                self.room = None;

                self.server.do_send(Leave {
                    id: self.id.clone(),
                    room: data.to_string(),
                });
            }
            "roomMessage" => {
                self.server.do_send(RoomMessage {
                    id: self.id.clone(),
                    name: self.name.clone().unwrap_or("".to_string()),
                    room: data["room"].to_string(),
                    message: data["message"].to_string(),
                });
            }
            "privateMessage" => {
                self.server.do_send(PrivateMessage {
                    emitter: data["emmiter"].to_string(),
                    receiver: data["receiver"].to_string(),
                    message: data["message"].to_string(),
                });
            }
            "profile" => {
                self.name = Some(data["name"].to_string());

                self.server.do_send(Profile {
                    id: self.id.clone(),
                    name: self.name.clone().unwrap(),
                });
            }
            _ => {}
        }
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Socket conected {}", self.id);

        // Start process on session start
        self.heartbeat(ctx);

        self.server.do_send(Connect {
            id: self.id.clone(),
            name: self.name.clone().unwrap_or("".to_string()),
            addr: ctx.address().recipient(),
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        // Leave room if joined to one
        if let Some(room) = &self.room {
            self.server.do_send(Leave {
                id: self.id.clone(),
                room: room.clone(),
            });
        }

        self.server.do_send(Disconnect {
            id: self.id.clone(),
        });

        ctx.terminate();

        log::info!("Stopped socket {}", self.id);
    }
}

impl Handler<Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0.dump());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        if let Ok(message) = msg {
            match message {
                ws::Message::Ping(_) => {
                    self.hb = Instant::now();
                    ctx.pong(b"pong");
                }
                ws::Message::Pong(_) => {
                    self.hb = Instant::now();
                }
                ws::Message::Text(text) => {
                    print!("");

                    let value: json::JsonValue = json::parse(&text).unwrap_or(json::Null);

                    if value == json::Null {
                        log::error!("Invalid message provided");
                        return;
                    }

                    let event: &str = value["event"].as_str().unwrap();
                    let data: json::JsonValue = value["data"].clone();

                    self.handle_events(event, data);
                }
                _ => {}
            }
        } else {
            log::error!("Protocol error: {}", msg.unwrap_err());
        }
    }
}
