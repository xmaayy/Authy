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

#[derive(Serialize)]
pub struct LoginResponse {
    pub refresh_token: String,
    pub access_token: String,
}

#[tokio::main]
async fn main() {
    let users = Arc::new(init_users());

    let create_user_route = warp::path!("create")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_user_handler);

    let list_route = warp::path!("list").and(warp::get()).and_then(list_handler);

    let login_route = warp::path!("login")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(login_handler);

    //let user_route = warp::path!("refresh")
    //    .and(warp::post())
    //    .and(with_auth(Role::Refresh))
    //    .and_then(());

    let refresh_route = warp::path!("refresh")
        .and(with_auth(Role::Refresh))
        .and_then(refresh_handler);

    let routes = login_route
        .or(refresh_route)
        .or(list_route)
        .or(create_user_route)
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

fn password_auth(
    body: LoginRequest,
) -> impl Filter<Extract = (String,), Error = Infallible> + Clone {
    warp::any().map(move || {
        user_database::password_login(body.clone().email, body.clone().pw)
            .unwrap()
            .clone()
    })
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

pub async fn login_handler(body: LoginRequest) -> WebResult<impl Reply> {
    let uid = user_database::password_login(body.clone().email, body.clone().pw)
        .unwrap()
        .clone();

    Ok(uid)
}

pub async fn refresh_handler(uid: String) -> WebResult<impl Reply> {
    let refresh_token =
        auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
    let access_token =
        auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?;
    Ok(reply::json(&LoginResponse {
        refresh_token,
        access_token,
    }))
}

pub async fn user_handler(uid: String) -> WebResult<impl Reply> {
    Ok(format!("Hello User {}", uid))
}

pub async fn admin_handler(uid: String) -> WebResult<impl Reply> {
    Ok(format!("Hello Admin {}", uid))
}

fn init_users() -> HashMap<String, User> {
    let mut map = HashMap::new();
    map.insert(
        String::from("1"),
        User {
            uid: String::from("1"),
            email: String::from("user@userland.com"),
            pw: String::from("1234"),
            role: String::from("User"),
        },
    );
    map.insert(
        String::from("2"),
        User {
            uid: String::from("2"),
            email: String::from("admin@adminaty.com"),
            pw: String::from("4321"),
            role: String::from("Admin"),
        },
    );
    map
}
