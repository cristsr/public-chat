use actix::{fut, prelude::*};
use actix_web_actors::ws;
use serde_json::Value;
use std::time::{Duration, Instant};

use crate::server;

#[derive(Default)]
pub struct WsSession {
    pub id: String,
    pub hb: Instant,
    pub name: Option(String),
    pub room: String,
    pub addr: Addr<server::ChatServer>,
}

impl WsSession {
    pub fn join_room(&mut self, room: &str) {}
}

impl Actor for WsSession {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        log::info!("Stopped");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        let message = match msg {
            Ok(message) => message,
            Err(err) => {
                ctx.stop();
                return;
            }
        };

        match message {
            ws::Message::Text(text) => {
                log::info!("Text: {}", text);

                // let json: MyObj = serde_json::from_slice::<MyObj>((&text).as_ref()).unwrap();

                let v: Value = serde_json::from_str(&text).unwrap();

                let event = v["event"].to_string()?;

                match event {
                    "join" => {
                        self.join_room(&v["data"].to_string()?);
                        log::info!("Join");
                    }
                    "leave" => {
                        log::info!("leave");
                    }
                    "asdf" => {
                        log::info!("asdf");
                    }
                }

                log::info!("event {} data {:?}", v["event"], v["data"]);
            }
            _ => {}
        }
    }
}
