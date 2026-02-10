use std::time::Duration;

const MSG_SIZE: usize = 1024;

struct MixItem {
    cipher_text: Vec<u8>,
    epochs_left: u32,
}

pub struct Mixer {
    queue: Vec<MixItem>,
    p: f64,
    epoch: Duration,
}
