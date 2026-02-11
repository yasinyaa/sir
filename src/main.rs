mod actors;
mod utils;

use std::time::Duration;

use actix::prelude::*;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;
use dotenv;
use env_logger;

use crate::actors::chat::ChatServer;
use crate::actors::mixer::Mixer;
use crate::actors::session::Session;

/* ---------------- WebSocket handler ---------------- */

async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    let session = Session::new(server.get_ref().clone());
    ws::start(session, &req, stream)
}

/* ---------------- main ---------------- */

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().expect("failed to find .env file");
    env_logger::init();

    let chat_server = ChatServer::new(None).start();

    let mixer = Mixer::new(1.0, Duration::from_secs(1), chat_server.clone()).start();

    chat_server.do_send(crate::actors::chat::SetMixer {
        mixer: Some(mixer.clone()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
