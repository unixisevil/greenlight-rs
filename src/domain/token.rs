use chrono::{DateTime, Duration, Utc};
use data_encoding::BASE32_NOPAD;
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::validator::Validator;

pub const SCOPE_ACTIVATION: &str = "activation";
pub const SCOPE_AUTHENTICATION: &str = "authentication";
pub const SCOPE_PASSWORDRESET: &str = "password-reset";

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Token {
    #[serde(rename = "token")]
    pub plain_text: String,

    #[serde(skip)]
    pub hash: Vec<u8>,

    #[serde(skip)]
    pub user_id: i64,

    #[serde(skip_deserializing)]
    pub expiry: DateTime<Utc>,

    #[serde(skip)]
    pub scope: &'static str,
}

impl Token {
    pub fn new(user_id: i64, ttl: Duration, scope: &'static str) -> Self {
        let mut rng = rand::thread_rng();
        let mut buf = [0u8; 16];
        rng.fill_bytes(&mut buf[..]);
        let token = BASE32_NOPAD.encode(&buf);
        //let token_hash = Sha256::digest(&token).to_vec();
        let token_hash  = Self::gen_hash(&token);

        Self {
            user_id,
            scope,
            expiry: Utc::now() + ttl,
            plain_text:  token,
            hash:  token_hash,
        }
    }

    pub fn gen_hash(plain_text: impl AsRef<str>)  ->  Vec<u8>  {
        Sha256::digest(plain_text.as_ref()).to_vec()
    }

    pub fn validate(v: &mut Validator,  plain_text: impl AsRef<str>) {
        v.check(!plain_text.as_ref().is_empty(),  "token",  "must be provided");
        v.check(plain_text.as_ref().len() == 26 , "token",  "must be 26 bytes long");
    }

}


