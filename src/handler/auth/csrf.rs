use axum::http::{header::COOKIE, HeaderMap, HeaderName};
use ring::rand::{SecureRandom, SystemRandom};
use tracing::warn;

use crate::handler::auth::extract_cookie_from_cookies;

pub const CSRF_COOKIE_NAME: &str = "_csrf_token=";
pub const CSRF_HEADER_NAME: HeaderName = HeaderName::from_static("x-csrf-token");

/// Generate 32 crypto random bytes as a 64-char hex string
pub fn generate_csrf_token() -> String {
    let rng = SystemRandom::new();
    let mut bytes = [0u8; 32];
    rng.fill(&mut bytes).expect("Failed to generate CSRF token");
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Build the Set-Cookie header value for the CSRF token
pub fn csrf_set_cookie_header(token: &str) -> String {
    format!(
        "{}{}; Secure; SameSite=Strict; Path=/",
        CSRF_COOKIE_NAME, token
    )
}

/// Build the Set-Cookie header value to clear the CSRF token (logout)
pub fn csrf_clear_cookie_header() -> String {
    "_csrf_token=; Secure; SameSite=Strict; Path=/; Max-Age=0".to_string()
}

pub fn verify_csrf_token(headers: &HeaderMap) -> bool {
    let cookie_val = headers
        .get(COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(|cookies| extract_cookie_from_cookies(cookies, CSRF_COOKIE_NAME));

    let header_val = headers
        .get(&CSRF_HEADER_NAME)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    match (cookie_val, header_val) {
        (Some(cookie), Some(header)) if cookie == header => true,
        _ => {
            warn!("CSRF token mismatch or missing");
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_generate_csrf_token_length() {
        let token = generate_csrf_token();
        assert_eq!(token.len(), 64);
    }

    #[test]
    fn test_generate_csrf_token_unique() {
        let token1 = generate_csrf_token();
        let token2 = generate_csrf_token();
        assert_ne!(token1, token2);
    }

    #[test]
    fn test_generate_csrf_token_hex_only() {
        let token = generate_csrf_token();
        assert!(token.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_csrf_set_cookie_header_format() {
        let token = "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4";
        let header = csrf_set_cookie_header(token);
        assert_eq!(
            header,
            "_csrf_token=a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4; Secure; SameSite=Strict; Path=/"
        );
    }

    #[test]
    fn test_csrf_clear_cookie_header_format() {
        let header = csrf_clear_cookie_header();
        assert_eq!(
            header,
            "_csrf_token=; Secure; SameSite=Strict; Path=/; Max-Age=0"
        );
    }

    #[test]
    fn test_verify_csrf_token_match() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str("_csrf_token=abc123").unwrap());
        headers.insert(
            CSRF_HEADER_NAME.clone(),
            HeaderValue::from_str("abc123").unwrap(),
        );
        assert!(verify_csrf_token(&headers));
    }

    #[test]
    fn test_verify_csrf_token_mismatch() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str("_csrf_token=abc123").unwrap());
        headers.insert(
            CSRF_HEADER_NAME.clone(),
            HeaderValue::from_str("xyz789").unwrap(),
        );
        assert!(!verify_csrf_token(&headers));
    }

    #[test]
    fn test_verify_csrf_token_missing_cookie() {
        let mut headers = HeaderMap::new();
        headers.insert(
            CSRF_HEADER_NAME.clone(),
            HeaderValue::from_str("abc123").unwrap(),
        );
        assert!(!verify_csrf_token(&headers));
    }

    #[test]
    fn test_verify_csrf_token_missing_header() {
        let mut headers = HeaderMap::new();
        headers.insert(COOKIE, HeaderValue::from_str("_csrf_token=abc123").unwrap());
        assert!(!verify_csrf_token(&headers));
    }

    #[test]
    fn test_verify_csrf_token_multiple_cookies() {
        let mut headers = HeaderMap::new();
        headers.insert(
            COOKIE,
            HeaderValue::from_str("token=jwt; _csrf_token=abc123; session=xyz").unwrap(),
        );
        headers.insert(
            CSRF_HEADER_NAME.clone(),
            HeaderValue::from_str("abc123").unwrap(),
        );
        assert!(verify_csrf_token(&headers));
    }
}
