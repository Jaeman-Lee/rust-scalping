use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn sign(secret_key: &str, message: &str) -> String {
    let mut mac =
        HmacSha256::new_from_slice(secret_key.as_bytes()).expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}

pub fn build_signed_query(secret_key: &str, params: &[(&str, &str)]) -> String {
    let query: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("&");
    let signature = sign(secret_key, &query);
    format!("{}&signature={}", query, signature)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_length() {
        let secret = "NhqPtmdSJYdKjVHjA7PZj4Mge3R5YNiP1e3UZjInClVN65XAbvqqM6A7H5fATj0j";
        let message = "symbol=LTCBTC&side=BUY&type=LIMIT&timeInForce=GTC&quantity=1&price=0.1&recvWindow=5000&timestamp=1499827319559";
        let sig = sign(secret, message);
        assert!(!sig.is_empty());
        assert_eq!(sig.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_sign_deterministic() {
        let secret = "test_secret";
        let message = "hello=world";
        let sig1 = sign(secret, message);
        let sig2 = sign(secret, message);
        assert_eq!(sig1, sig2);
    }

    #[test]
    fn test_sign_different_secrets() {
        let message = "hello=world";
        let sig1 = sign("secret1", message);
        let sig2 = sign("secret2", message);
        assert_ne!(sig1, sig2);
    }

    #[test]
    fn test_build_signed_query() {
        let secret = "test_secret";
        let params = [("symbol", "BTCUSDT"), ("side", "BUY")];
        let query = build_signed_query(secret, &params);
        assert!(query.starts_with("symbol=BTCUSDT&side=BUY&signature="));
        assert_eq!(query.matches("signature=").count(), 1);
    }

    #[test]
    fn test_build_signed_query_empty_params() {
        let secret = "test_secret";
        let params: [(&str, &str); 0] = [];
        let query = build_signed_query(secret, &params);
        assert!(query.starts_with("&signature="));
    }

    #[test]
    fn test_sign_hex_chars_only() {
        let sig = sign("key", "message");
        assert!(sig.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
