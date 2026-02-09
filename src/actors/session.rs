use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;

use crate::actors::chat::{ChatMessage, ChatServer, Connect, Disconnect};

pub struct Session {
    pub id: Uuid,
    pub server: Addr<ChatServer>,
}

impl Session {
    pub fn new(server: Addr<ChatServer>) -> Self {
        Self {
            id: Uuid::new_v4(),
            server,
        }
    }
}

impl Actor for Session {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address().recipient();

        self.server.do_send(Connect {
            session_id: self.id,
            addr,
        });
    }

    fn stopped(&mut self, _: &mut Self::Context) {
        self.server.do_send(Disconnect {
            session_id: self.id,
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Session {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text)) => {
                // encrypted payload from client
                self.server.do_send(ChatMessage {
                    from: self.id,
                    payload: text.to_string(),
                });
            }
            Ok(ws::Message::Ping(p)) => ctx.pong(&p),
            Ok(ws::Message::Close(_)) => ctx.stop(),
            _ => {}
        }
    }
}

impl Handler<ChatMessage> for Session {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, ctx: &mut Self::Context) {
        // encrypted payload back to client
        ctx.text(msg.payload);
    }
}
