use actix::prelude::*;
use actix_web_actors::ws;
use std::time::{Duration, Instant};

use crate::message::{Connect, Disconnect, Join, Leave, Message, RoomMessage};
use crate::server;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub id: String,
    pub name: String,
    pub room: Option<String>,
    pub hb: Instant,
    pub server: Addr<server::ChatServer>,
}

impl WsSession {
    /// Send ping to client every 5 seconds
    pub fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |session, ctx| {
            if Instant::now().duration_since(session.hb) > CLIENT_TIMEOUT {
                log::info!("Client {} is gone, disconnecting!", session.id);

                // Remove from server
                session.server.do_send(Disconnect {
                    id: session.id.clone(),
                    room: session.room.clone(),
                });

                // Stop actor
                ctx.stop();

                //
                return;
            }

            ctx.ping(b"ping");
        });
    }
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Started, id: {}", self.id);

        // Start process on session start
        self.heartbeat(ctx);

        self.server.do_send(Connect {
            id: self.id.clone(),
            addr: ctx.address().recipient(),
        });
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::info!("Stopped");
        self.server.do_send(Disconnect {
            id: self.id.clone(),
            room: self.room.clone(),
        });

        ctx.stop();
    }
}

impl Handler<Message> for WsSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) -> Self::Result {
        log::info!("Message received {:?}", msg);
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(message) => match message {
                ws::Message::Ping(_) => {
                    self.hb = Instant::now();
                    ctx.pong(b"pong");
                }
                ws::Message::Pong(_) => {
                    self.hb = Instant::now();
                }
                ws::Message::Text(text) => {
                    log::info!("\n===================");

                    let value: json::JsonValue = json::parse(&text).unwrap();

                    let event: &str = value["event"].as_str().unwrap();
                    let data: json::JsonValue = value["data"].clone();

                    log::info!("Event: {}", event);
                    log::info!("Data: {}", data);

                    match event {
                        "joinRoom" => {
                            self.server.do_send(Join {
                                id: self.id.clone(),
                                name: self.name.clone(),
                                room: data.to_string(),
                            });
                        }
                        "leaveRoom" => {
                            self.server.do_send(Leave {
                                id: self.id.clone(),
                                room: data.to_string(),
                            });
                        }
                        "roomMessage" => {
                            self.server.do_send(RoomMessage {
                                id: self.id.clone(),
                                name: self.name.clone(),
                                room: data["room"].to_string(),
                                message: data["message"].to_string(),
                            });
                        }
                        "changeName" => {
                            self.name = data.to_string();
                            
                            ctx.text(
                                object! {
                                   event: "changeName",
                                   data: self.name.clone()
                                }
                                .dump(),
                            );
                        }
                        _ => {}
                    }
                }
                _ => {}
            },
            Err(e) => {
                log::error!("{:?}", e);

                ctx.text(
                    object! {
                        error: e.to_string()
                    }
                    .dump(),
                );

                ctx.stop();
                return;
            }
        };
    }
}
