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
use password_hash::{PasswordHash, PasswordVerifier};
use regex::Regex;
use tracing::{debug, error, warn};
use urlencoding::decode as url_decode;

/// Take request body String from POST login to get email and password
fn process_login_body(body: &str) -> Option<(String, String)> {
    // Initialize fields
    let mut email = String::new();
    let mut password = String::new();

    let req_fields: Vec<&str> = body.split("&").collect();
    for req_field in req_fields {
        let (key, value) = req_field.split_once("=").unwrap();
        let value_decoded = url_decode(value).unwrap();
        debug!("Request field key/value {:?}/{:?}", key, value_decoded);
        match key {
            "login_email" => email = value_decoded.to_string(),
            "login_password" => password = value_decoded.to_string(),
            _ => {
                warn!("Unrecognized key/value: {:?}/{:?}", key, value_decoded);
                continue;
            }
        }
    }
    Some((email, password))
}

/// Take HeaderMap from GET login to produce user agent and JWT token
pub fn process_login_header(header: HeaderMap) -> Option<(String, String)> {
    // Initialize fields
    let mut user_agent = String::new();
    let mut token = String::new();

    for (key, value) in header.iter() {
        match *key {
            USER_AGENT => user_agent = value.to_str().unwrap().to_string(),
            COOKIE => {
                let (_, tkn) = value.to_str().unwrap().split_once("token=").unwrap();
                token = tkn.to_string()
            }
            _ => {
                debug!("Unrecognized key/value: {:?}/{:?}", key, value);
                continue;
            }
        }
    }

    Some((user_agent, token))
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
    let argon2_algo: &[&dyn PasswordVerifier] = &[&Argon2::default()];

    if password_hash
        .verify_password(argon2_algo, password)
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
