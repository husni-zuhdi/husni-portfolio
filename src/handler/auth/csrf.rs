use axum::http::{header::COOKIE, HeaderMap, HeaderName};
use ring::rand::{SecureRandom, SystemRandom};
use tracing::warn;

const CSRF_COOKIE_NAME: &str = "_csrf_token=";
const CSRF_HEADER_NAME: HeaderName = HeaderName::from_static("x-csrf-token");

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

/// Extract CSRF token from Cookie header
fn extract_csrf_from_cookies(cookie_header: &str) -> Option<String> {
    cookie_header
        .split("; ")
        .find(|c| c.starts_with(CSRF_COOKIE_NAME))
        .map(|c| c[CSRF_COOKIE_NAME.len()..].to_string())
}

pub fn verify_csrf_token(headers: &HeaderMap) -> bool {
    let cookie_val = headers
        .get(COOKIE)
        .and_then(|v| v.to_str().ok())
        .and_then(extract_csrf_from_cookies);

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
