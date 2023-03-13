use data_encoding::HEXUPPER;
use ring::digest::{Context, SHA256};

pub fn sha256_digest(buffer: &Vec<u8>) -> String {
    let mut context = Context::new(&SHA256);
    if buffer.len() == 0 {
        return String::from("");
    }
    context.update(&buffer);
    let digest = context.finish();
    let signature = HEXUPPER.encode(digest.as_ref());
    signature
}
