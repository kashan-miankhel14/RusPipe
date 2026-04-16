use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Verify GitHub's X-Hub-Signature-256 header.
/// header_value is the raw header string: "sha256=<hex>"
pub fn verify_github_signature(secret: &str, body: &[u8], header_value: &str) -> bool {
    let Some(hex_sig) = header_value.strip_prefix("sha256=") else {
        return false;
    };
    let Ok(sig_bytes) = hex::decode(hex_sig) else {
        return false;
    };
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC accepts any key size");
    mac.update(body);
    mac.verify_slice(&sig_bytes).is_ok()
}
