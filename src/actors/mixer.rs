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
    tick: u64,
}

impl Mixer {
    pub fn new(p: f64, epoch: Duration, chat: Addr<crate::actors::chat::ChatServer>) -> Self {
        Self {
            queue: Vec::new(),
            p,
            epoch,
            chat,
            tick: 0,
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
        log::info!("Mixer started: epoch={:?}, p={}", self.epoch, self.p);

        ctx.run_interval(self.epoch, |_, ctx| {
            ctx.address().do_send(Flush);
        });
    }
}

/* -------- Handlers -------- */

impl Handler<Enqueue> for Mixer {
    type Result = ();

    fn handle(&mut self, msg: Enqueue, _: &mut Context<Self>) {
        if msg.cipher_text.len() != MSG_SIZE {
            log::warn!(
                "Mixer dropped msg: expected {} bytes, got {}",
                MSG_SIZE,
                msg.cipher_text.len()
            );
            return;
        }

        let epochs = self.sample_epoch();

        log::info!(
            "Mixer enqueue: size={}, epochs_left={}",
            msg.cipher_text.len(),
            epochs
        );

        self.queue.push(MixItem {
            cipher_text: msg.cipher_text,
            epochs_left: epochs,
        });
    }
}

impl Handler<Flush> for Mixer {
    type Result = ();

    fn handle(&mut self, _: Flush, _: &mut Context<Self>) {
        self.tick += 1;

        let mut ready = Vec::new();

        for (_idx, item) in self.queue.iter_mut().enumerate() {
            if item.epochs_left > 0 {
                item.epochs_left -= 1;
            }

            if item.epochs_left == 0 {
                ready.push(item.cipher_text.clone());
            }
        }

        // Keep only messages still waiting
        self.queue.retain(|item| item.epochs_left > 0);

        if ready.is_empty() {
            return;
        }

        randomize_msgs_order(&mut ready);

        log::info!("Mixer FLUSH → sending batch size={}", ready.len());

        self.chat.do_send(MixedBatch { messages: ready });
    }
}
