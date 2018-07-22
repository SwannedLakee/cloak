use ring::{digest, hmac};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum HashFunction {
    SHA1,
    SHA256,
    SHA384,
    SHA512,
    SHA512_256,
}

#[derive(Debug)]
pub struct OTP {
    secret_key: Vec<u8>,
    counter: u64,
    totp: bool,
    output_len: usize,
    output_base: Vec<u8>,
    hash_function: HashFunction,
}

impl OTP {
    pub fn new(
        secret_key: Vec<u8>,
        totp: bool,
        hash_function: HashFunction,
        counter: Option<u64>,
        output_len: Option<usize>,
    ) -> OTP {
        let counter = match counter {
            Some(c) => c,
            None => 0 as u64,
        };
        let output_len = match output_len {
            Some(len) => len,
            None => 6,
        };
        OTP {
            secret_key,
            counter,
            totp,
            output_len,
            output_base: "0123456789".to_owned().into_bytes(),
            hash_function,
        }
    }

    pub fn generate(&self) -> String {
        let counter = self.get_counter();
        let message: [u8; 8] = [
            ((counter >> 56) & 0xff) as u8,
            ((counter >> 48) & 0xff) as u8,
            ((counter >> 40) & 0xff) as u8,
            ((counter >> 32) & 0xff) as u8,
            ((counter >> 24) & 0xff) as u8,
            ((counter >> 16) & 0xff) as u8,
            ((counter >> 8) & 0xff) as u8,
            (counter & 0xff) as u8,
        ];
        let signing_key = match self.hash_function {
            HashFunction::SHA1 => hmac::SigningKey::new(&digest::SHA1, &self.secret_key),
            HashFunction::SHA256 => hmac::SigningKey::new(&digest::SHA256, &self.secret_key),
            HashFunction::SHA384 => hmac::SigningKey::new(&digest::SHA384, &self.secret_key),
            HashFunction::SHA512 => hmac::SigningKey::new(&digest::SHA512, &self.secret_key),
            HashFunction::SHA512_256 => {
                hmac::SigningKey::new(&digest::SHA512_256, &self.secret_key)
            }
        };
        let digest = hmac::sign(&signing_key, &message);
        self.encode_digest(digest.as_ref())
    }

    fn encode_digest(&self, digest: &[u8]) -> String {
        let offset = (*digest.last().unwrap() & 0xf) as usize;
        let snum: u32 = ((u32::from(digest[offset]) & 0x7f) << 24)
            | ((u32::from(digest[offset + 1]) & 0xff) << 16)
            | ((u32::from(digest[offset + 2]) & 0xff) << 8)
            | (u32::from(digest[offset + 3]) & 0xff);
        let base = self.output_base.len() as u32;
        let hotp_code = snum % base.pow(self.output_len as u32);
        let mut code = hotp_code.to_string();
        while code.len() < self.output_len {
            code = "0".to_owned() + &code;
        }
        code
    }

    fn get_counter(&self) -> u64 {
        if self.totp {
            let timestamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u64;
            timestamp / 30
        } else {
            self.counter
        }
    }
}