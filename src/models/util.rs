//! Utility stuff for the models, not exported
//!

use diesel::QueryResult;
use rsa::{
    pkcs1::EncodeRsaPrivateKey,
    pkcs8::{EncodePublicKey, LineEnding},
    RsaPrivateKey, RsaPublicKey,
};

use base64::prelude::*;

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
