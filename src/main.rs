mod actors;
mod routes;
mod utils;

use std::time::Duration;

use actix::prelude::*;
use actix_web::error::ErrorInternalServerError;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;

use crate::actors::chat::ChatServer;
use crate::actors::mixer::Mixer;
use crate::actors::session::Session;
use crate::routes::auth;
use crate::utils::redis::RedisService;

async fn get_all_messages(redis: web::Data<RedisService>) -> Result<HttpResponse, Error> {
    let messages = redis.get_all_messages().map_err(ErrorInternalServerError)?;

    Ok(HttpResponse::Ok().json(messages))
}

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

    let redis = RedisService::from_env().expect("failed to initialize redis service from .env");

    let chat_server = ChatServer::new(None).start();

    let mixer = Mixer::new(
        0.2,
        Duration::from_secs(3),
        chat_server.clone(),
        redis.clone(),
    )
    .start();

    chat_server.do_send(crate::actors::chat::SetMixer {
        mixer: Some(mixer.clone()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .app_data(web::Data::new(redis.clone()))
            .route("/messages", web::get().to(get_all_messages))
            .route("/ws", web::get().to(ws_handler))
            .route("/auth/challenge", web::post().to(auth::sign_in))
            .route("/auth/verify", web::post().to(auth::verify))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
