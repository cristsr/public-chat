#[macro_use]
extern crate json;
use actix::{Actor, Addr};
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use std::time::Instant;
use uuid::Uuid;

mod config;
mod message;
mod server;
mod session;

async fn index() -> impl Responder {
    "Hello world!"
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<impl Responder, Error> {
    let session = session::WsSession {
        id: Uuid::new_v4().to_string(),
        hb: Instant::now(),
        room: None,
        name: "".to_string(),
        server: srv.get_ref().clone(),
    };

    log::info!("New session: {:?}", session);

    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("Starting server...");

    let chat_server = server::ChatServer::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .route("/", web::get().to(index))
            .route("/ws", web::get().to(ws_route))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
