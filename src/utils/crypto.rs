use base64::{Engine as _, engine::general_purpose};
use chrono::{Duration, Utc};
use rand::RngCore;
use rand::rngs::OsRng;

pub(crate) const CHALLENGE_SIZE: usize = 32;
pub(crate) const CHALLENGE_TTL_SECS: i64 = 60;

pub fn generate_challenge() -> (String, u64) {
    let mut buf = [0u8; CHALLENGE_SIZE];
    OsRng.fill_bytes(&mut buf);

    let challenge_b64 = general_purpose::STANDARD.encode(buf);

    let expires_at = (Utc::now() + Duration::seconds(CHALLENGE_TTL_SECS)).timestamp() as u64;

    (challenge_b64, expires_at)
}
