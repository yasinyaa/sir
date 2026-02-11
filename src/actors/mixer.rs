use actix::prelude::*;
use rand::random;
use std::time::Duration;

use crate::actors::chat::MixedBatch;
use crate::utils::randomize::randomize_msgs_order;

const MSG_SIZE: usize = 1024;

/* -------- Internal -------- */

struct MixItem {
    cipher_text: Vec<u8>,
    epochs_left: u32,
}

/* -------- Actor -------- */

pub struct Mixer {
    queue: Vec<MixItem>,
    p: f64,
    epoch: Duration,
    chat: Addr<crate::actors::chat::ChatServer>,
}

impl Mixer {
    pub fn new(p: f64, epoch: Duration, chat: Addr<crate::actors::chat::ChatServer>) -> Self {
        Self {
            queue: Vec::new(),
            p,
            epoch,
            chat,
        }
    }

    fn sample_epoch(&self) -> u32 {
        let mut epochs = 0;
        while random::<f64>() > self.p {
            epochs += 1;
        }
        epochs
    }
}

/* -------- Messages -------- */

#[derive(Message)]
#[rtype(result = "()")]
pub struct Enqueue {
    pub cipher_text: Vec<u8>,
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct Flush;

/* -------- Actor impl -------- */

impl Actor for Mixer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.run_interval(self.epoch, |_, ctx| {
            ctx.address().do_send(Flush);
        });
    }
}

/* -------- Handlers -------- */

// impl Handler<Enqueue> for Mixer {
//     type Result = ();

//     fn handle(&mut self, msg: Enqueue, _: &mut Context<Self>) {
//         if msg.cipher_text.len() != MSG_SIZE {
//             log::warn!(
//                 "Dropped msg: expected {} bytes, got {}",
//                 MSG_SIZE,
//                 msg.cipher_text.len()
//             );
//             return;
//         }

//         self.queue.push(MixItem {
//             cipher_text: msg.cipher_text,
//             epochs_left: self.sample_epoch(),
//         });
//     }
// }
//
impl Handler<Enqueue> for Mixer {
    type Result = ();

    fn handle(&mut self, msg: Enqueue, _: &mut Context<Self>) {
        log::info!("Mixer enqueue len={}", msg.cipher_text.len());

        if msg.cipher_text.len() != MSG_SIZE {
            log::warn!("Dropped message: wrong size");
            return;
        }

        self.queue.push(MixItem {
            cipher_text: msg.cipher_text,
            epochs_left: self.sample_epoch(),
        });
    }
}

impl Handler<Flush> for Mixer {
    type Result = ();

    fn handle(&mut self, _: Flush, _: &mut Context<Self>) {
        let mut ready = Vec::new();

        for item in &mut self.queue {
            if item.epochs_left == 0 {
                ready.push(item.cipher_text.clone());
            } else {
                item.epochs_left -= 1;
            }
        }

        self.queue.retain(|item| item.epochs_left > 0);

        if ready.is_empty() {
            return;
        }

        randomize_msgs_order(&mut ready);

        self.chat.do_send(MixedBatch { messages: ready });
    }
}
