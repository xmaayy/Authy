use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct User {
    pub uid: String,
    pub email: String,
    pub pw: String,
    pub role: String,
}

#[derive(Deserialize, Clone)]
pub struct LoginRequest {
    pub email: String,
    pub pw: String,
}

/// This is what is sent back to a user / client when they authenticate
/// with a username and password or with a valid refresh token.
#[derive(Serialize)]
pub struct LoginResponse {
    pub refresh_token: String,
    pub access_token: String,
}

/// An access request is made to the server when a client recieves
/// a request from a user to access a resource. Right now there is
/// no RBAC implemented, but this is where you would put the resource
/// identifier if that becomes a requirement
#[derive(Deserialize, Clone)]
pub struct AccessRequest {
    pub uid: String,
}
