use serde::{Deserialize, Serialize};

/// User
/// Requiered to have:
/// - auto-generated user id
/// - email
/// - hashed_password
pub struct User {
    pub id: String,
    pub email: String,
    pub hashed_password: String,
}

/// Session
/// Active client session for specific User and device. Contains:
/// - auto-generated session id
/// - user id
/// - auto-generated sesion token
/// - auto-generated expire date time in UTC+0
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub token: String,
    pub expire: String,
}

/// UserCommandStatus
/// Status of User Command Operations:
/// - Stored
/// - Updated
/// - Deleted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum UserCommandStatus {
    Stored,
    Updated,
    Deleted,
}

/// SessionCommandStatus
/// Status of Session Command Operations:
/// - Stored
/// - Deleted
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionCommandStatus {
    Stored,
    Deleted,
}

/// Claims
/// JWT claims
/// Reference: https://github.com/Keats/jsonwebtoken?tab=readme-ov-file#claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub exp: usize,
    pub iat: usize,
}
