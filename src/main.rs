use actix::Addr;
use actix_web::{middleware::Logger, web, App, Error, HttpRequest, HttpServer, Responder};
use actix_web_actors::ws;
use log::log;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::Instant;
use uuid::Uuid;

mod server;
mod session;

async fn configure_ws(
    req: HttpRequest,
    stream: web::Payload,
    srv: web::Data<Addr<server::ChatServer>>,
) -> Result<impl Responder, Error> {
    let session = session::WsSession {
        id: Uuid::new_v4(),
        hb: Instant::now(),
        room: "Main".to_owned(),
        name: None,
        addr: srv.get_ref().clone(),
    };

    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    log::info!("Starting server...");

    let app_state = Arc::new(AtomicUsize::new(0));

    let ws_server = server::ChatServer::new(app_state.clone());

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
