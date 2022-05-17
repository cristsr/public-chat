use actix::prelude::*;
use actix_web_actors::ws;
use serde_json::{json, Value};
use std::time::{Duration, Instant};

use crate::message::{Connect, Disconnect, Join, Leave, Message, RoomMessage};
use crate::server;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Debug)]
pub struct WsSession {
    pub id: String,
    pub hb: Instant,
    pub name: Option<String>,
    pub room: Option<String>,
    pub server: Addr<server::ChatServer>,
}

impl WsSession {
    /**
     * Send ping to client every 5 seconds
     */
    pub fn heartbeat(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |session, ctx| {
            if Instant::now().duration_since(session.hb) > CLIENT_TIMEOUT {
                log::info!("Client {} is gone, disconnecting!", session.id);

                // Remove from server
                session.server.do_send(Disconnect {
                    id: session.id.clone(),
                    room: session.room.clone().unwrap_or("".to_string()),
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
            room: self.room.clone().unwrap_or("".to_string()),
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
        let message = match msg {
            Ok(message) => message,
            Err(e) => {
                log::error!("{:?}", e);
                ctx.stop();
                return;
            }
        };

        match message {
            ws::Message::Ping(msg) => {
                self.hb = Instant::now();
                ctx.pong(b"pong");
            }
            ws::Message::Pong(_) => {
                self.hb = Instant::now();
            }
            ws::Message::Text(text) => {
                // let json: MyObj = serde_json::from_slice::<MyObj>((&text).as_ref()).unwrap();
                // parse json
                let value: Value = match serde_json::from_str(&text) {
                    Ok(value) => value,
                    Err(e) => {
                        log::error!("{}", e);
                        ctx.text(json!({ "error": e.to_string() }).to_string());
                        return;
                    }
                };

                let event: &str = value["event"].as_str().unwrap();
                let data: Value = value["data"].clone();

                match event {
                    "join" => {
                        let room: String = String::from(data.as_str().unwrap_or(""));

                        log::info!("{} join room {}", self.id, room);

                        self.server.do_send(Join {
                            id: self.id.clone(),
                            name: self.name.clone().unwrap_or("RandomName".to_string()),
                            room: room.clone(),
                        });
                    }
                    "leave" => {
                        let room = String::from(data.as_str().unwrap_or(""));
                        let id = self.id.clone();

                        self.server.do_send(Leave { id, room });
                    }
                    "roomMessage" => {
                        let room = String::from(data["room"].as_str().unwrap_or(""));
                        let message = String::from(data["msg"].as_str().unwrap_or(""));

                        self.server.do_send(RoomMessage {
                            id: self.id.clone(),
                            name: self.name.clone().unwrap_or(String::from("")),
                            room,
                            message,
                        });
                    }

                    "name" => {
                        let name = String::from(data.as_str().unwrap_or(""));
                        self.name = Some(name.clone());

                        ctx.text(json!({ "event": "name", "data": name }).to_string());
                    }
                    _ => {}
                }

                log::info!("event {} data {:?}", value["event"], value["data"]);
            }
            _ => {}
        }
    }
}
