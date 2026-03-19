use std::fmt;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::Sha256;

const KEY_SIZE: usize = 128;
const SALT_SIZE: usize = 4;
const HMAC_SLICE_SIZE: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortSignedId {
    name: String,
    sign: Vec<u8>,
}

impl ShortSignedId {
    pub fn new<R: Rng + ?Sized>(
        name: String,
        key: &[u8; KEY_SIZE],
        rng: &mut R,
    ) -> anyhow::Result<ShortSignedId> {
        let mut salt = [0u8; SALT_SIZE];
        rng.try_fill_bytes(&mut salt)?;

        let sign = calc_sign(&name, key, &salt)?;

        Ok(ShortSignedId { name, sign })
    }

    pub fn parse(id: &str) -> anyhow::Result<ShortSignedId> {
        let mut parts = id.splitn(2, '.');
        let name = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid ID format"))?
            .to_string();
        let sign = parts
            .next()
            .ok_or_else(|| anyhow::anyhow!("Invalid ID format"))?
            .as_bytes();
        let sign = BASE64_URL_SAFE_NO_PAD.decode(sign)?;
        Ok(ShortSignedId { name, sign })
    }

    pub fn verify(&self, key: &[u8; KEY_SIZE]) -> anyhow::Result<bool> {
        if self.sign.len() != SALT_SIZE + HMAC_SLICE_SIZE {
            return Ok(false);
        }
        let salt = &self.sign[..SALT_SIZE];
        let expected_sign = calc_sign(&self.name, key, salt)?;
        Ok(self.sign == expected_sign)
    }
}

impl Into<String> for ShortSignedId {
    fn into(self) -> String {
        let sign_b64 = BASE64_URL_SAFE_NO_PAD.encode(&self.sign);
        format!("{}.{}", self.name, sign_b64)
    }
}

impl fmt::Display for ShortSignedId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<String>::into(self.clone()))
    }
}

fn calc_sign(name: &str, key: &[u8; KEY_SIZE], salt: &[u8]) -> anyhow::Result<Vec<u8>> {
    let mut hmac = Hmac::<Sha256>::new_from_slice(key)?;
    hmac.update(name.as_bytes());
    hmac.update(salt);
    let hmac_sign = hmac.finalize().into_bytes();

    let mut sign = Vec::new();
    sign.extend_from_slice(salt);
    sign.extend_from_slice(&hmac_sign[..HMAC_SLICE_SIZE]);
    Ok(sign)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand_core::OsRng;

    #[test]
    fn test_short_signed_id() {
        let key = [0u8; KEY_SIZE];
        let id = ShortSignedId::new("test".to_string(), &key, &mut OsRng).unwrap();
        println!("ID: {}", id);

        assert!(id.verify(&key).unwrap());

        let parsed_id = ShortSignedId::parse(&id.to_string()).unwrap();
        println!("Parsed ID: {}", parsed_id);
        assert_eq!(id, parsed_id);
        assert!(parsed_id.verify(&key).unwrap());
    }

    #[test]
    fn name_modified_id() {
        let key = [0u8; KEY_SIZE];
        let id = ShortSignedId::new("test".to_string(), &key, &mut OsRng).unwrap();

        let mut modified_id = id.clone();
        modified_id.name = "modified".to_string();
        assert!(!modified_id.verify(&key).unwrap());
    }

    #[test]
    fn sign_modified_id() {
        let key = [0u8; KEY_SIZE];
        let id = ShortSignedId::new("test".to_string(), &key, &mut OsRng).unwrap();

        let mut modified_id = id.clone();
        modified_id.sign[0] ^= 0xFF;
        assert!(!modified_id.verify(&key).unwrap());
    }
}
