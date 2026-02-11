use actix::prelude::*;
use log;
use std::collections::HashMap;
use uuid::Uuid;

use crate::actors::mixer::{Enqueue, Mixer};

/// Opaque ciphertext broadcast to sessions
#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct ChatMessage {
    pub payload: Vec<u8>,
}

/// Mixer → ChatServer
#[derive(Message)]
#[rtype(result = "()")]
pub struct MixedBatch {
    pub messages: Vec<Vec<u8>>,
}

/// Session connects
#[derive(Message)]
#[rtype(result = "()")]
pub struct Connect {
    pub session_id: Uuid,
    pub addr: Recipient<ChatMessage>,
}

/// Session disconnects
#[derive(Message)]
#[rtype(result = "()")]
pub struct Disconnect {
    pub session_id: Uuid,
}

/// Set Mixer
#[derive(Message)]
#[rtype(result = "()")]
pub struct SetMixer {
    pub mixer: Option<Addr<Mixer>>,
}

pub struct ChatServer {
    sessions: HashMap<Uuid, Recipient<ChatMessage>>,
    mixer: Option<Addr<Mixer>>,
}

impl ChatServer {
    pub fn new(mixer: Option<Addr<Mixer>>) -> Self {
        Self {
            sessions: HashMap::new(),
            mixer,
        }
    }

    pub fn set_mixer(&mut self, mixer: Option<Addr<Mixer>>) {
        match mixer {
            Some(addr) => {
                log::info!("Mixer attached to ChatServer");
                self.mixer = Some(addr);
            }
            None => {
                log::warn!("Attempted to set mixer to None");
                self.mixer = None;
            }
        }
    }

    fn broadcast(&self, payload: Vec<u8>) {
        let msg = ChatMessage { payload };
        for addr in self.sessions.values() {
            let _ = addr.do_send(msg.clone());
        }
    }
}

impl Actor for ChatServer {
    type Context = Context<Self>;
}

/* -------- Handlers -------- */

impl Handler<Connect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Connect, _: &mut Context<Self>) {
        log::info!("Session connected: {}", msg.session_id);
        self.sessions.insert(msg.session_id, msg.addr);
    }
}

impl Handler<Disconnect> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: Disconnect, _: &mut Context<Self>) {
        self.sessions.remove(&msg.session_id);
    }
}

/// Incoming ciphertext → mixer
// impl Handler<ChatMessage> for ChatServer {
//     type Result = ();

//     fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
//         if let Some(mixer) = &self.mixer {
//             mixer.do_send(Enqueue {
//                 cipher_text: msg.payload,
//             });
//         } else {
//             log::warn!("Received message before mixer attached");
//         }
//     }
// }
//
impl Handler<ChatMessage> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: ChatMessage, _: &mut Context<Self>) {
        log::info!(
            "ChatServer received message len={}, mixer_attached={}",
            msg.payload.len(),
            self.mixer.is_some()
        );

        if let Some(mixer) = &self.mixer {
            mixer.do_send(Enqueue {
                cipher_text: msg.payload,
            });
        }
    }
}

/// Mixer output → broadcast
impl Handler<MixedBatch> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: MixedBatch, _: &mut Context<Self>) {
        for cipher_text in msg.messages {
            self.broadcast(cipher_text);
        }
    }
}

impl Handler<SetMixer> for ChatServer {
    type Result = ();

    fn handle(&mut self, msg: SetMixer, _ctx: &mut Context<Self>) {
        self.set_mixer(msg.mixer);
    }
}
