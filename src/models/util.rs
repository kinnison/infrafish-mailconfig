//! Utility stuff for the models, not exported
//!

use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher};
use diesel::QueryResult;
use rsa::{
    pkcs1::EncodeRsaPrivateKey,
    pkcs8::{EncodePublicKey, LineEnding},
    rand_core::OsRng,
    RsaPrivateKey, RsaPublicKey,
};

use base64::prelude::*;

use super::MailUser;

pub fn create_dkim_pair() -> QueryResult<(String, String)> {
    let mut rng = rand::thread_rng();
    const BITS: usize = 2048;

    let privkey = RsaPrivateKey::new(&mut rng, BITS)
        .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))?;
    let pubkey = RsaPublicKey::from(&privkey);

    let privkey = privkey
        .to_pkcs1_pem(LineEnding::LF)
        .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))?;

    let pubkey = pubkey
        .to_public_key_der()
        .map_err(|e| diesel::result::Error::QueryBuilderError(Box::new(e)))?;

    let pubkey = BASE64_STANDARD.encode(pubkey.as_bytes());

    Ok((privkey.to_string(), pubkey))
}

/// An extension to be used by routes to determine access control
/// If this extension isn't present that means that the user didn't
/// supply a token.  if they supplied a token and it was bad then
/// we return an error instead.
#[derive(Debug, Clone)]
pub struct Authorisation {
    token: String,
    user: i32,
    username: String,
    superuser: bool,
}

impl Authorisation {
    pub fn new(token: String, user: &MailUser) -> Self {
        Self {
            token,
            user: user.id,
            username: user.username.clone(),
            superuser: user.superuser,
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn user(&self) -> i32 {
        self.user
    }

    pub fn username(&self) -> &str {
        &self.username
    }

    pub fn superuser(&self) -> bool {
        self.superuser
    }
}

pub fn encode_password(password: &str) -> String {
    (if let Some(rest) = password.strip_prefix("{ARGON2ID}") {
        PasswordHash::new(rest).map(|h| h.serialize())
    } else {
        let salt = SaltString::generate(&mut OsRng);
        PasswordHash::new(
            &Argon2::default()
                .hash_password(password.as_bytes(), &salt)
                .expect("Unable to hash password")
                .to_string(),
        )
        .map(|h| h.serialize())
    })
    .map(|h| format!("{{ARGON2ID}}{h}"))
    .unwrap_or_else(|_| String::from(password))
}
