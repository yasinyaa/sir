use std::env;
use std::fmt::Write;

use redis::{Client, Commands, RedisResult};

pub const REDIS_MESSAGES_KEY: &str = "messages";

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub fn from_env() -> RedisResult<Self> {
        let url = env::var("REDIS_URL").map_err(|err| {
            redis::RedisError::from(std::io::Error::other(format!(
                "REDIS_URL must be set in .env: {err}"
            )))
        })?;

        Self::new(&url)
    }

    pub fn new(url: &str) -> RedisResult<Self> {
        let client = Client::open(url)?;
        Ok(Self { client })
    }

    pub fn save_challenge(&self, challenge: &str, key: &str, ttl_secs: u64) -> RedisResult<usize> {
        let mut conn = self.client.get_connection()?;
        conn.set_ex(key, challenge, ttl_secs)?
    }

    pub fn get_challenge(&self, key: &str) -> RedisResult<Option<String>> {
        let mut conn = self.client.get_connection()?;
        conn.get(key)
    }

    pub fn delete_challenge(&self, key: &str) -> RedisResult<()> {
        let mut conn = self.client.get_connection()?;
        let _: usize = conn.del(key)?;
        Ok(())
    }

    pub fn save_message(&self, message: &[u8]) -> RedisResult<usize> {
        let mut conn = self.client.get_connection()?;
        let encoded = encode_hex(message);
        conn.rpush(REDIS_MESSAGES_KEY, encoded)
    }

    pub fn get_all_messages(&self) -> RedisResult<Vec<String>> {
        let mut conn = self.client.get_connection()?;
        conn.lrange(REDIS_MESSAGES_KEY, 0, -1)
    }
}

fn encode_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut out, "{b:02x}");
    }
    out
}
