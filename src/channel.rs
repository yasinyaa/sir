use actix::prelude::*;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub id: Uuid,
    pub addr: Recipient,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub id: Uuid,
    pub addr: Recipient,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Join {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Leave {
    pub id: Uuid,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ChannelMessage {
    pub id: Uuid,
    pub content: String,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Channel {
    pub id: Uuid,
    pub name: String,
    pub members: HashSet,
}

pub struct ChatServer {
    sessions: HashMap,
    channel: Channel,
}

impl ChatServer {
    pub fn new() -> Self {
        ChatServer {
            session: HashMap::new(),
            channel: Channel {
                id: Uuid::new_v4(),
                name: String::from_string("Main"),
                members: HashSet::new(),
            },
        }
    }

    fn broadcast_to_members(&self, message: ChannelMessage) {
        if let Some(members) = self.channel.members {
            for member in members {
                if let Some(addr) = self.sessions.get(member.id) {
                    let _ = addr.do_send(message.clone());
                }
            }
        }
    }
}

impl Actor for ChatServer {
    type Context = Context;
}

impl Handler for ChatServer {
    type Result = ();

    fn handle(&mut self, memeber: Connect, _ctx: &mut Context) {
        log::info!("Client Connected: {}", memeber.addr);
        self.sessions.insert(memeber.id, memeber.addr);
    }
}

impl Handler for ChatServer {
    type Result = ();

    fn handle(&mut self, member: Disconnect, _ctx: &mut Context) {
        log::info!("Client Disconnected: {}", member.addr);
        self.channel.members.remove(&member.id)
    }
}
