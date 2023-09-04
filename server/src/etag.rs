use std::sync::Arc;
use const_random::const_random;

pub struct Etag {}

impl Etag {
    const ALPHANUMERIC:&'static str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    const ETAG_BYTES: [u8; 32] = const_random!([u8; 32]);

    pub fn get() -> Arc<String> {
        let mut etag: Vec<u8> = Vec::new();

        for etag_byte in Self::ETAG_BYTES {
            let char_index = etag_byte % Self::ALPHANUMERIC.len() as u8;
            etag.push(Self::ALPHANUMERIC.as_bytes()[char_index as usize]);
        }

        let etag = std::str::from_utf8(&etag).unwrap();
        let etag = Arc::new(String::from(etag));

        return etag;
    }

}
