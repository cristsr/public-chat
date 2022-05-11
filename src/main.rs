use std::any::Any;
use std::iter::Map;
use actix::{fut, prelude::*};
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use actix_web_actors::ws::ProtocolError;
use json::JsonValue;
use json::object::Object;
use log::log;
use serde::{Deserialize, Serialize};
use serde_json::Value;

enum Data {
    Room(String),
    Close,
}


#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    event: String,
    data: String
}

#[derive(Default)]
struct WsSession {
    id: usize,
}

impl WsSession {

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

impl StreamHandler<Result<ws::Message, ProtocolError>> for WsSession {
    fn handle(&mut self, msg: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
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
                        log::info!("Join");
                    }
                    "leave" => {
                        log::info!("leave");
                    }
                }

                log::info!("event {} data {:?}", v["event"], v["data"]);
            }
            _ => {}
        }
    }
}

async fn configure_ws(req: HttpRequest, stream: web::Payload) -> Result<impl Responder, Error> {
    ws::start(WsSession::default(), &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()>{
  env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

  log::info!("Starting server...");

  HttpServer::new(move || {
    App::new()
      .service(web::resource("/ws").to(configure_ws))
      .wrap(Logger::default())
  })
    .workers(2)
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
