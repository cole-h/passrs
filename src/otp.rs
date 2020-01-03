use data_encoding::BASE32_NOPAD;
use data_encoding::HEXLOWER_PERMISSIVE;
use ring::hmac;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Result;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum HashAlgorithm {
    Sha1,
    Sha256,
    Sha512,
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

        pub fn base32_secret<S>(&mut self, secret: S) -> Result<&mut $t>
        where
            S: AsRef<[u8]>,
        {
            let secret = secret.as_ref();
            self.key = BASE32_NOPAD.decode(secret)?;
            Ok(self)
        }

        pub fn ascii_secret<S>(&mut self, secret: S) -> &mut $t
        where
            S: AsRef<[u8]>,
        {
            let secret = secret.as_ref();
            self.secret(secret)
        }

        pub fn hex_secret<S>(&mut self, secret: S) -> &mut $t
        where
            S: AsRef<[u8]>,
        {
            let secret = secret.as_ref();
            let hex = HEXLOWER_PERMISSIVE
                .decode(secret)
                .expect("Secret was invalid hex");
            self.key = hex;
            self
        }

        pub fn output_len(&mut self, output_len: usize) -> &mut $t {
            self.output_len = output_len;
            self
        }

        pub fn hash_function(&mut self, algo: HashAlgorithm) -> &mut $t {
            self.algo = algo;
            self
        }
    }
}

#[derive(Debug)]
pub struct HOTPBuilder {
    key: Vec<u8>,
    counter: u64,
    output_len: usize,
    algo: HashAlgorithm,
}

#[allow(dead_code)]
impl HOTPBuilder {
    pub fn new() -> HOTPBuilder {
        HOTPBuilder {
            key: vec![],
            counter: 0,
            output_len: 6,
            algo: HashAlgorithm::Sha1,
        }
    }

    otp_builder!(HOTPBuilder);

    pub fn counter(&mut self, counter: u64) -> &mut HOTPBuilder {
        self.counter = counter;
        self
    }

    pub fn output_length(&mut self, len: usize) -> &mut HOTPBuilder {
        self.output_len = len;
        self
    }

    pub fn algorithm(&mut self, algo: HashAlgorithm) -> &mut HOTPBuilder {
        self.algo = algo;
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
        let offset = (hmac_result.last().unwrap() & 0xf) as usize;

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

#[derive(Debug)]
pub struct TOTPBuilder {
    key: Vec<u8>,
    counter: u64,
    timestamp_offset: i64,
    output_len: usize,
    algo: HashAlgorithm,
    period: u64,
}

#[allow(dead_code)]
impl TOTPBuilder {
    pub fn new() -> TOTPBuilder {
        TOTPBuilder {
            key: vec![],
            counter: 0,
            timestamp_offset: 0,
            output_len: 6,
            algo: HashAlgorithm::Sha1,
            period: 30,
        }
    }

    otp_builder!(TOTPBuilder);

    pub fn timestamp(&mut self, timestamp: i64) -> &mut TOTPBuilder {
        let current_timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.timestamp_offset = timestamp - current_timestamp;
        self
    }

    pub fn counter(&mut self, counter: u64) -> &mut TOTPBuilder {
        self.counter = counter;
        self
    }

    pub fn output_length(&mut self, len: usize) -> &mut TOTPBuilder {
        self.output_len = len;
        self
    }

    pub fn algorithm(&mut self, algo: HashAlgorithm) -> &mut TOTPBuilder {
        self.algo = algo;
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

/// For more informatio see RFC6238: https://tools.ietf.org/html/rfc6238
impl TOTP {
    fn counter(&self) -> u64 {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
            + self.timestamp_offset;
        let timestamp = timestamp as u64;
        timestamp / self.period
    }

    pub fn generate(&self) -> String {
        let counter = self.counter();
        let hotp = HOTPBuilder::new()
            .secret(self.key.clone())
            .counter(counter)
            .output_length(self.output_len)
            .algorithm(self.algo)
            .build();

        hotp.generate()
    }
}

/// Tests taken from boringauth: https://docs.rs/boringauth/0.9.0/src/boringauth/oath/totp.rs.html
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_base32key_full() {
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];
        let key_base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_owned();

        let totp = TOTPBuilder::new()
            .base32_secret(&key_base32)
            .unwrap()
            .timestamp(1111111109)
            .period(70)
            .output_length(8)
            .algorithm(HashAlgorithm::Sha256)
            .build();

        assert_eq!(totp.key, key);
        assert_eq!(totp.period, 70);
        assert_eq!(totp.output_len, 8);

        let code = totp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "04696041");
    }
    #[test]
    fn test_totp_base32key_simple() {
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];
        let key_base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_owned();

        let totp = TOTPBuilder::new()
            .base32_secret(&key_base32)
            .unwrap()
            .build();

        assert_eq!(totp.key, key);
        assert_eq!(totp.output_len, 6);
        assert_eq!(totp.algo, HashAlgorithm::Sha1);

        let code = totp.generate();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_hotp_base32key_simple() {
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];
        let key_base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_owned();

        let hotp = HOTPBuilder::new()
            .base32_secret(&key_base32)
            .unwrap()
            .algorithm(HashAlgorithm::Sha256)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 0);
        assert_eq!(hotp.output_len, 6);
        assert_eq!(hotp.algo, HashAlgorithm::Sha256);

        let code = hotp.generate();
        assert_eq!(code.len(), 6);
        assert_eq!(code, "875740");
    }

    #[test]
    fn test_hotp_base32key_full() {
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];
        let key_base32 = "GEZDGNBVGY3TQOJQGEZDGNBVGY3TQOJQ".to_owned();

        let hotp = HOTPBuilder::new()
            .base32_secret(&key_base32)
            .unwrap()
            .counter(5)
            .output_length(8)
            .algorithm(HashAlgorithm::Sha512)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 5);
        assert_eq!(hotp.output_len, 8);
        assert_eq!(hotp.algo, HashAlgorithm::Sha512);

        let code = hotp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "16848329");
    }

    #[test]
    fn test_totp_kexkey_simple() {
        let key_hex = "3132333435363738393031323334353637383930".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let totp = TOTPBuilder::new().hex_secret(&key_hex).build();

        assert_eq!(totp.key, key);
        assert_eq!(totp.output_len, 6);
        assert_eq!(totp.algo, HashAlgorithm::Sha1);

        let code = totp.generate();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_totp_hexkey_full() {
        let key_hex = "3132333435363738393031323334353637383930".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let totp = TOTPBuilder::new()
            .hex_secret(&key_hex)
            .timestamp(1111111109)
            .period(70)
            .output_len(8)
            .hash_function(HashAlgorithm::Sha256)
            .build();
        assert_eq!(totp.key, key);
        assert_eq!(totp.period, 70);
        assert_eq!(totp.output_len, 8);

        let code = totp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "04696041");
    }

    #[test]
    fn test_hotp_hexkey_simple() {
        let key_hex = "3132333435363738393031323334353637383930".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let hotp = HOTPBuilder::new()
            .hex_secret(&key_hex)
            .algorithm(HashAlgorithm::Sha256)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 0);
        assert_eq!(hotp.output_len, 6);
        assert_eq!(hotp.algo, HashAlgorithm::Sha256);

        let code = hotp.generate();
        assert_eq!(code.len(), 6);
        assert_eq!(code, "875740");
    }

    #[test]
    fn test_hotp_hexkey_full() {
        let key_hex = "3132333435363738393031323334353637383930".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let hotp = HOTPBuilder::new()
            .hex_secret(&key_hex)
            .counter(5)
            .output_len(8)
            .algorithm(HashAlgorithm::Sha512)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 5);
        assert_eq!(hotp.output_len, 8);
        assert_eq!(hotp.algo, HashAlgorithm::Sha512);

        let code = hotp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "16848329");
    }

    #[test]
    fn test_totp_asciikey_simple() {
        let key_ascii = "12345678901234567890".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let totp = TOTPBuilder::new().ascii_secret(&key_ascii).build();

        assert_eq!(totp.key, key);
        assert_eq!(totp.output_len, 6);
        assert_eq!(totp.algo, HashAlgorithm::Sha1);

        let code = totp.generate();
        assert_eq!(code.len(), 6);
    }

    #[test]
    fn test_totp_asciikey_full() {
        let key_ascii = "12345678901234567890".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let totp = TOTPBuilder::new()
            .ascii_secret(&key_ascii)
            .timestamp(1111111109)
            .period(70)
            .output_len(8)
            .algorithm(HashAlgorithm::Sha256)
            .build();
        assert_eq!(totp.key, key);
        assert_eq!(totp.period, 70);
        assert_eq!(totp.output_len, 8);

        let code = totp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "04696041");
    }

    #[test]
    fn test_hotp_asciikey_simple() {
        let key_ascii = "12345678901234567890".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let hotp = HOTPBuilder::new()
            .ascii_secret(&key_ascii)
            .algorithm(HashAlgorithm::Sha256)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 0);
        assert_eq!(hotp.output_len, 6);
        assert_eq!(hotp.algo, HashAlgorithm::Sha256);

        let code = hotp.generate();
        assert_eq!(code.len(), 6);
        assert_eq!(code, "875740");
    }

    #[test]
    fn test_hotp_asciikey_full() {
        let key_ascii = "12345678901234567890".to_owned();
        let key = vec![
            49, 50, 51, 52, 53, 54, 55, 56, 57, 48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 48,
        ];

        let hotp = HOTPBuilder::new()
            .ascii_secret(&key_ascii)
            .counter(5)
            .output_len(8)
            .algorithm(HashAlgorithm::Sha256)
            .build();

        assert_eq!(hotp.key, key);
        assert_eq!(hotp.counter, 5);
        assert_eq!(hotp.output_len, 8);
        assert_eq!(hotp.algo, HashAlgorithm::Sha256);

        let code = hotp.generate();
        assert_eq!(code.len(), 8);
        assert_eq!(code, "89697997");
    }
}
