//! One-Time Password generation
//!
//! # otp
//!
//! This module houses the implementation of RFC6238 and RFC4226 for use in
//! generating Time-based One-Time Passwords.
//!
//! Requires the `otp` feature to be enabled (enabled by default).

use std::time::{SystemTime, UNIX_EPOCH};

use data_encoding::BASE32_NOPAD;
use ring::hmac;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha512,
}

impl Default for HashAlgorithm {
    fn default() -> HashAlgorithm {
        HashAlgorithm::Sha1
    }
}

macro_rules! otp_builder {
    ($t:ty) => {
        pub fn secret<V>(&mut self, secret: V) -> &mut $t
        where
            V: AsRef<[u8]>,
        {
            self.key = secret.as_ref().to_vec();

            self
        }

        pub fn base32_secret<S>(&mut self, secret: S) -> &mut $t
        where
            S: AsRef<[u8]>,
        {
            let secret = secret.as_ref();
            let secret = secret.to_ascii_uppercase();
            self.key = BASE32_NOPAD
                .decode(&secret)
                .expect("Secret was not valid base32");

            self
        }

        pub fn output_len(&mut self, output_len: usize) -> &mut $t {
            self.output_len = output_len;

            self
        }

        pub fn algorithm(&mut self, algo: HashAlgorithm) -> &mut $t {
            self.algo = algo;

            self
        }
    };
}

#[derive(Debug, Default)]
pub struct HOTPBuilder {
    key: Vec<u8>,
    counter: u64,
    output_len: usize,
    algo: HashAlgorithm,
}

impl HOTPBuilder {
    otp_builder!(HOTPBuilder);

    pub fn counter(&mut self, counter: u64) -> &mut HOTPBuilder {
        self.counter = counter;

        self
    }

    pub fn build(&self) -> HOTP {
        HOTP {
            key: self.key.clone(),
            counter: self.counter,
            output_len: self.output_len,
            algo: self.algo,
        }
    }
}

#[derive(Debug)]
pub struct HOTP {
    key: Vec<u8>,
    counter: u64,
    output_len: usize,
    algo: HashAlgorithm,
}

/// See RFC4226 for more information: https://tools.ietf.org/html/rfc4226
impl HOTP {
    pub fn generate(&self) -> String {
        let counter = self.counter;

        // "The Key (K), the Counter (C), and Data values are hashed high-order byte first."
        //     So, we need to convert the counter to big endian
        let moving_factor: [u8; 8] = [
            ((counter >> 56) & 0xff) as u8,
            ((counter >> 48) & 0xff) as u8,
            ((counter >> 40) & 0xff) as u8,
            ((counter >> 32) & 0xff) as u8,
            ((counter >> 24) & 0xff) as u8,
            ((counter >> 16) & 0xff) as u8,
            ((counter >> 8) & 0xff) as u8,
            (counter & 0xff) as u8,
        ];

        let key = match self.algo {
            HashAlgorithm::Sha1 => hmac::Key::new(hmac::HMAC_SHA1_FOR_LEGACY_USE_ONLY, &self.key),
            HashAlgorithm::Sha256 => hmac::Key::new(hmac::HMAC_SHA256, &self.key),
            HashAlgorithm::Sha512 => hmac::Key::new(hmac::HMAC_SHA512, &self.key),
        };

        // HS = HMAC-SHA(K, C)
        let hmac_result = hmac::sign(&key, &moving_factor);
        let hmac_result = hmac_result.as_ref();

        // `offset` is in the range 0..15, inclusive
        let offset = (hmac_result
            .last()
            .expect("hmac_result didn't have a last element")
            & 0xf) as usize;

        // Convert the hmac_result (S) to a number in 0..2^{32}-1 (0x7fff_ffff)
        let snum: u32 = ((u32::from(hmac_result[offset]) & 0x7f) << 24)
            | ((u32::from(hmac_result[offset + 1]) & 0xff) << 16)
            | ((u32::from(hmac_result[offset + 2]) & 0xff) << 8)
            | (u32::from(hmac_result[offset + 3]) & 0xff);

        // `code` is Snum mod 10^Digit, where Snum is the truncated hash and Digit
        //     is the length of the generated code
        let code = snum % 10_u32.pow(self.output_len as u32);
        format!("{:0width$}", code, width = self.output_len)
    }
}

#[derive(Debug, Default)]
pub struct TOTPBuilder {
    key: Vec<u8>,
    counter: u64,
    timestamp_offset: i64,
    output_len: usize,
    algo: HashAlgorithm,
    period: u64,
}

impl TOTPBuilder {
    otp_builder!(TOTPBuilder);

    pub fn timestamp(&mut self, timestamp: i64) -> &mut TOTPBuilder {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get duration since UNIX_EPOCH")
            .as_secs() as i64;
        self.timestamp_offset = timestamp - current_timestamp;

        self
    }

    pub fn counter(&mut self, counter: u64) -> &mut TOTPBuilder {
        self.counter = counter;

        self
    }

    pub fn period(&mut self, period: u64) -> &mut TOTPBuilder {
        self.period = period;

        self
    }

    pub fn build(&self) -> TOTP {
        TOTP {
            key: self.key.clone(),
            timestamp_offset: self.timestamp_offset,
            counter: self.counter,
            output_len: self.output_len,
            algo: self.algo,
            period: self.period,
        }
    }
}

#[derive(Debug)]
pub struct TOTP {
    key: Vec<u8>,
    timestamp_offset: i64,
    counter: u64,
    output_len: usize,
    algo: HashAlgorithm,
    period: u64,
}

/// For more information see RFC6238: https://tools.ietf.org/html/rfc6238
impl TOTP {
    fn counter(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Couldn't get duration since UNIX_EPOCH")
            .as_secs() as i64
            + self.timestamp_offset;
        let timestamp = timestamp as u64;

        timestamp / self.period
    }

    pub fn generate(&self) -> String {
        let counter = self.counter();
        let hotp = HOTPBuilder::default()
            .secret(self.key.clone())
            .counter(counter)
            .output_len(self.output_len)
            .algorithm(self.algo)
            .build();

        hotp.generate()
    }
}
