use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: Uuid,
    pub addr: Recipient<ChatMessage>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: Uuid,
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub from: Uuid,
    pub payload: String, // encrypted blob
}

pub struct ChatServer {
    sessions: HashMap<Uuid, Recipient<ChatMessage>>,
    members: HashSet<Uuid>, // single global room
}

impl ChatServer {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            members: HashSet::new(),
        }
    }

    fn broadcast(&self, msg: ChatMessage) {
        for member_id in &self.members {
            if let Some(addr) = self.sessions.get(member_id) {
                let _ = addr.do_send(msg.clone());
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        self.sessions.insert(msg.session_id, msg.addr);
        self.members.insert(msg.session_id);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.session_id);
        self.members.remove(&msg.session_id);
    }
}

impl Handler<ChatMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
        self.broadcast(msg);
    }
}
