use argon2::{self, Config};
use auth::{with_auth, Role};
use error::Error::*;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{reject, reply, Filter, Rejection, Reply};

mod auth;
mod error;
mod user_database;

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>;

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

#[tokio::main]
async fn main() {
    let create_user_route = warp::path!("create")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_user_handler);

    let list_route = warp::path!("list").and(warp::get()).and_then(list_handler);

    let access_auth_route = warp::path!("access")
        .and(warp::post())
        .and(with_auth(Role::Access))
        .and(warp::body::json())
        .and_then(validate_user_access);

    let login_route = warp::path!("login")
        .and(warp::post())
        .and(password_auth())
        .and_then(login_handler);

    let refresh_route = warp::path!("refresh")
        .and(with_auth(Role::Refresh))
        .and_then(refresh_handler);

    let routes = login_route
        .or(access_auth_route)
        .or(refresh_route)
        .or(list_route)
        .or(create_user_route)
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn password_auth() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32)
        .and(warp::body::json())
        .and_then(|body: LoginRequest| async move {
            match user_database::password_login(body.clone().email, body.clone().pw) {
                Ok(uid) => Ok(uid),
                Err(err) => Err(reject::custom(error::Error::NoPermissionError)),
            }
        })
}

///
pub async fn validate_user_access(uid: String, body: AccessRequest) -> WebResult<impl Reply> {
    // We really dont need to allow much more than this for a small
    // authorization request
    match uid == body.uid {
        true => Ok(uid),
        false => Err(reject::custom(error::Error::NoPermissionError)),
    }
}

pub async fn create_user_handler(body: LoginRequest) -> WebResult<impl Reply> {
    match user_database::create_user(body.email, body.pw) {
        Ok(uid) => {
            println!("Created user with UID {:?}", uid);
            // IMPLEMENT ACTUALLY USING TOKENS NOW
        }
        Err(custom_err) => {
            return Err(reject::custom(custom_err));
        }
    }

    Ok(format!("'access':'aksjdljas', 'refresh':'asdasd'"))
}

pub async fn list_handler() -> WebResult<impl Reply> {
    user_database::list_users();
    Ok(format!("Printed"))
}

/// Called from the /login endpoint with a username and password in the json payload,
/// this endpoint will create a new access and refresh token for the client.
pub async fn login_handler(uid: String) -> WebResult<impl Reply> {
    println!("UID: {:?}", uid);
    let refresh_token = auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
    let access_token = auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?;
    Ok(reply::json(&LoginResponse {
        refresh_token,
        access_token,
    }))
}

/// Called from the /refresh endpoint with your refresh bearer token. When called with
/// the proper authorization it will return a new set of access and refresh tokens which
/// are valid for the configured time period.
pub async fn refresh_handler(uid: String) -> WebResult<impl Reply> {
    let refresh_token = auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
    let access_token = auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?;
    Ok(reply::json(&LoginResponse {
        refresh_token,
        access_token,
    }))
}
