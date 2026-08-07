pub mod csrf;
pub mod displays;
pub mod operations;

use crate::{model::auth::Claims, utils::remove_whitespace};
use argon2::Argon2;
use axum::http::{
    header::{COOKIE, USER_AGENT},
    HeaderMap,
};
use jsonwebtoken::{
    decode as jwt_decode, encode as jwt_encode, DecodingKey, EncodingKey, Header, Validation,
};
use password_hash::{phc::PasswordHash, PasswordVerifier};
use regex::Regex;
use tracing::info;
use tracing::{debug, error, warn};
use urlencoding::decode as url_decode;

/// Name of the JWT auth cookie as it appears in the `Cookie` header.
const JWT_COOKIE_NAME: &str = "token=";

/// Take request body String from POST login to get email and password
fn process_login_body(body: &str) -> Option<(String, String)> {
    // Initialize fields
    let mut email = String::new();
    let mut password = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = url_decode(value).unwrap();
        match key {
            "login_email" => email = value_decoded.to_string(),
            "login_password" => password = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
            }
        }
    }
    Some((email, password))
}

/// Extract a cookie value by name from a `Cookie` header.
///
/// The header is split on `"; "` and each segment is matched with
/// `starts_with(cookie_name)`. Matching per-segment (instead of searching for a
/// substring) avoids false hits, e.g. `token=` must not match the
/// `_csrf_token=` cookie.
///
/// Returns `None` when the cookie is absent.
pub fn extract_cookie_from_cookies(cookie_header: &str, cookie_name: &str) -> Option<String> {
    cookie_header
        .split("; ")
        .find(|c| c.starts_with(cookie_name))
        .map(|c| c[cookie_name.len()..].to_string())
}

/// Verify the `token` cookie holds a valid JWT signed with `jwt_secret`.
///
/// Returns `true` only when the JWT is present, well-formed, and not expired.
/// Missing, empty, or invalid tokens return `false` (resulting in a 401).
/// The `token` cookie is read via [`extract_cookie_from_cookies`].
pub fn is_auth_verified(header: HeaderMap, jwt_secret: &str) -> bool {
    let mut user_agent = String::new();
    let mut token = String::new();

    for (key, value) in header.iter() {
        match *key {
            USER_AGENT => user_agent = value.to_str().unwrap().to_string(),
            COOKIE => {
                let tkn = extract_cookie_from_cookies(value.to_str().unwrap(), JWT_COOKIE_NAME);
                match tkn {
                    Some(v) => token = v,
                    None => {
                        debug!("No token in cookies");
                        continue;
                    }
                };
            }
            _ => {
                debug!("Unrecognized key/value: {:?}/{:?}", key, value);
            }
        }
    }

    info!("User Agent: {} and JWT processed", user_agent);
    if !verify_jwt(&token, jwt_secret) {
        info!("Unauthorized access.");
        return false;
    }
    true
}

/// sanitize_email
/// Remove whitespace and check email pattern of an email
/// Return sanitized email or None
fn sanitize_email(email: &str) -> Option<String> {
    let non_whitespace_email = remove_whitespace(email);
    // Reference: https://regexr.com/3e48o
    // Before @, allow words (alphanumeric and numbers), ''-',  and '.'
    // Must contain '@'
    // After @, allow words (alphanumeric and numbers), '-', and must ended with '.'
    // After ., allow 2-4 alphanumeric, numbers, and '-'
    //let pattern = Regex::new(r"^\[\w-\.\]+@(\[\w-\]+\.)+\[\w-\]{2,4}$").unwrap();
    let pattern = Regex::new(r"^.*@.*\..*$").unwrap();
    let matched = pattern.find(&non_whitespace_email);
    if matched.is_none() {
        warn!("Email {} doesn't meet regex pattern", email);
        return None;
    }

    if matched.unwrap().as_str() != email {
        warn!(
            "Email {} is different than matched pattern {}",
            email,
            matched.unwrap().as_str()
        );
        return None;
    }
    Some(matched.unwrap().as_str().to_string())
}

/// sanitize_password
/// Remove whitespace.
/// TODO: think about it later
fn sanitize_password(password: &str) -> String {
    remove_whitespace(password)
}

/// is_password_match
/// Compare password from user with hashed_passwrod in the DB
fn is_password_match(password: &str, hashed_passwrod: &str) -> bool {
    let password_hash = PasswordHash::new(hashed_passwrod).expect("Invalid password hash");
    let argon2_algo: &dyn PasswordVerifier<PasswordHash> = &Argon2::default();

    if argon2_algo
        .verify_password(password.as_ref(), &password_hash)
        .is_err()
    {
        error!("Password is not matched!");
        false
    } else {
        true
    }
}

/// create_jwt
/// Create JWT Claim and token
fn create_jwt(secret: &str) -> Option<String> {
    let now = chrono::Utc::now().timestamp() as usize;
    let three_hour_in_s = 10800_usize;
    let my_claims = Claims {
        exp: now + three_hour_in_s,
        iat: now,
    };
    match jwt_encode(
        &Header::default(),
        &my_claims,
        &EncodingKey::from_secret(secret.as_ref()),
    ) {
        Ok(token) => Some(token),
        Err(e) => {
            error!("Failed to create JWT Token. {:?}", e);
            None
        }
    }
}

/// verify_jwt
/// Return bool of verified JWT
pub fn verify_jwt(token: &str, secret: &str) -> bool {
    if token.is_empty() {
        debug!("JWT is empty. Skip JWT verification.");
        return false;
    }

    let token = jwt_decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    );
    match token {
        Ok(_) => true,
        Err(e) => {
            warn!("Failed to verify JWT Token. {:?}", e);
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handler::auth::csrf::CSRF_COOKIE_NAME;
    use axum::http::HeaderValue;

    const SECRET: &str = "test-secret";

    fn insert_cookie(headers: &mut HeaderMap, cookie: &str) {
        if !cookie.is_empty() {
            headers.insert(COOKIE, HeaderValue::from_str(cookie).unwrap());
        }
    }

    #[test]
    fn test_extract_csrf_and_jwt_from_cookies_found() {
        let cookies = "_csrf_token=abc123; token=jwt456";
        let csrf = extract_cookie_from_cookies(cookies, CSRF_COOKIE_NAME);
        let jwt = extract_cookie_from_cookies(cookies, JWT_COOKIE_NAME);
        assert_eq!(csrf, Some("abc123".to_string()));
        assert_eq!(jwt, Some("jwt456".to_string()));
    }

    #[test]
    fn test_extract_cookies_token_first() {
        let cookies = "token=jwt456; _csrf_token=abc123";
        let csrf = extract_cookie_from_cookies(cookies, CSRF_COOKIE_NAME);
        let jwt = extract_cookie_from_cookies(cookies, JWT_COOKIE_NAME);
        assert_eq!(csrf, Some("abc123".to_string()));
        assert_eq!(jwt, Some("jwt456".to_string()));
    }

    #[test]
    fn test_extract_csrf_from_cookies_missing() {
        let cookie = "token=jwt456";
        let csrf = extract_cookie_from_cookies(cookie, CSRF_COOKIE_NAME);
        let jwt = extract_cookie_from_cookies(cookie, JWT_COOKIE_NAME);
        assert_eq!(csrf, None);
        assert_eq!(jwt, Some("jwt456".to_string()));
    }

    #[test]
    fn test_extract_csrf_from_cookies_empty() {
        let cookie = "";
        let csrf = extract_cookie_from_cookies(cookie, CSRF_COOKIE_NAME);
        let jwt = extract_cookie_from_cookies(cookie, JWT_COOKIE_NAME);
        assert_eq!(csrf, None);
        assert_eq!(jwt, None);
    }

    #[test]
    fn test_extract_csrf_from_cookies_first_position() {
        let cookie = "_csrf_token=abc123";
        let csrf = extract_cookie_from_cookies(cookie, CSRF_COOKIE_NAME);
        let jwt = extract_cookie_from_cookies(cookie, JWT_COOKIE_NAME);
        assert_eq!(jwt, None);
        assert_eq!(csrf, Some("abc123".to_string()));
    }

    #[test]
    fn test_extract_cookies_prefix_safe() {
        let cookies = "token_abc=xyz; _csrf_token=def";
        let jwt = extract_cookie_from_cookies(cookies, JWT_COOKIE_NAME);
        let csrf = extract_cookie_from_cookies(cookies, CSRF_COOKIE_NAME);
        assert_eq!(jwt, None);
        assert_eq!(csrf, Some("def".to_string()));
    }

    #[test]
    fn test_create_jwt_structure() {
        let token = create_jwt(SECRET).expect("should create a JWT");
        assert_eq!(token.split('.').count(), 3);
    }

    #[test]
    fn test_verify_jwt_valid_token() {
        let token = create_jwt(SECRET).unwrap();
        assert!(verify_jwt(&token, SECRET));
    }

    #[test]
    fn test_verify_jwt_empty_token() {
        assert!(!verify_jwt("", SECRET));
    }

    #[test]
    fn test_verify_jwt_garbage_token() {
        assert!(!verify_jwt("not.a.jwt", SECRET));
    }

    #[test]
    fn test_verify_jwt_wrong_secret() {
        let token = create_jwt(SECRET).unwrap();
        assert!(!verify_jwt(&token, "different-secret"));
    }

    #[test]
    fn test_verify_jwt_expired_token() {
        let now = chrono::Utc::now().timestamp() as usize;
        let expired_claims = Claims {
            exp: now.saturating_sub(3600),
            iat: now.saturating_sub(7200),
        };
        let token = jwt_encode(
            &Header::default(),
            &expired_claims,
            &EncodingKey::from_secret(SECRET.as_ref()),
        )
        .unwrap();
        assert!(!verify_jwt(&token, SECRET));
    }

    #[test]
    fn test_verify_jwt_tampered_token() {
        let token = create_jwt(SECRET).unwrap();
        let mut bytes = token.into_bytes();
        let last = bytes.last_mut().unwrap();
        *last = if *last == b'X' { b'Y' } else { b'X' };
        let tampered = String::from_utf8(bytes).unwrap();
        assert!(!verify_jwt(&tampered, SECRET));
    }

    #[test]
    fn test_is_auth_verified_valid_token_first() {
        let token = create_jwt(SECRET).unwrap();
        let mut headers = HeaderMap::new();
        insert_cookie(&mut headers, &format!("token={token}; _csrf_token=abc123"));
        assert!(is_auth_verified(headers, SECRET));
    }

    #[test]
    fn test_is_auth_verified_csrf_first() {
        let token = create_jwt(SECRET).unwrap();
        let mut headers = HeaderMap::new();
        insert_cookie(&mut headers, &format!("_csrf_token=abc123; token={token}"));
        assert!(is_auth_verified(headers, SECRET));
    }

    #[test]
    fn test_is_auth_verified_missing_token_cookie() {
        let mut headers = HeaderMap::new();
        insert_cookie(&mut headers, "_csrf_token=abc123");
        assert!(!is_auth_verified(headers, SECRET));
    }

    #[test]
    fn test_is_auth_verified_no_cookie_header() {
        let headers = HeaderMap::new();
        assert!(!is_auth_verified(headers, SECRET));
    }

    #[test]
    fn test_is_auth_verified_garbage_token() {
        let mut headers = HeaderMap::new();
        insert_cookie(&mut headers, "token=garbage; _csrf_token=abc123");
        assert!(!is_auth_verified(headers, SECRET));
    }

    #[test]
    fn test_is_auth_verified_wrong_secret() {
        let token = create_jwt(SECRET).unwrap();
        let mut headers = HeaderMap::new();
        insert_cookie(&mut headers, &format!("token={token}; _csrf_token=abc123"));
        assert!(!is_auth_verified(headers, "different-secret"));
    }
}
