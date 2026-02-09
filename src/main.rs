mod actors;

use actix::prelude::*;
use actix_web::{App, Error, HttpRequest, HttpResponse, HttpServer, web};
use actix_web_actors::ws;

use crate::actors::chat::ChatServer;
use crate::actors::session::Session;

async fn ws_handler(
    req: HttpRequest,
    stream: web::Payload,
    server: web::Data<Addr<ChatServer>>,
) -> Result<HttpResponse, Error> {
    let session = Session::new(server.get_ref().clone());
    ws::start(session, &req, stream)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let chat_server = ChatServer::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(chat_server.clone()))
            .route("/ws", web::get().to(ws_handler))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
