use auth::{with_auth, Role};
use error::Error::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use warp::{reject, reply, Filter, Rejection, Reply};
use argon2::{self, Config};
use rusqlite::Connection;

mod user_database;
mod auth;
mod error;

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

#[derive(Deserialize)]
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

    let login_route = warp::path!("create")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_user_handler);


    let refresh_route = warp::path!("login")
        .and(warp::post())
        .and(with_users(users.clone()))
        .and(warp::body::json())
        .and_then(refresh_handler);

    //let user_route = warp::path!("refresh")
    //    .and(with_auth(Role::Refresh))
    //    .and_then(());

    //let refresh_route = warp::path!("refresh")
    //    .and(warp::post())
    //    .and(with_auth(Role::User))
    //    .and(with_users(users.clone()))
    //    .and_then(login_handler);

    let routes = login_route
        .or(refresh_route)
    //    .or(admin_route)
        .recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}



fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

pub async fn create_user_handler(body: LoginRequest) -> WebResult<impl Reply> {

    user_database::create_user(body.email, body.pw);
    Ok(format!("Created User"))
}




pub async fn login_handler(users: Users, body: LoginRequest) -> WebResult<impl Reply> {
    match users
        .iter()
        .find(|(_uid, user)| user.email == body.email && user.pw == body.pw)
    {
        Some((uid, user)) => {
            let refresh_token = auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
            let access_token = auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?; 
            Ok(reply::json(&LoginResponse { refresh_token, access_token }))
        }
        None => Err(reject::custom(WrongCredentialsError))
    }
}



pub async fn refresh_handler(users: Users, body: LoginRequest) -> WebResult<impl Reply> {
    match users
        .iter()
        .find(|(_uid, user)| user.email == body.email && user.pw == body.pw)
    {
        Some((uid, user)) => {
            let refresh_token = auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
            let access_token = auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?; 
            Ok(reply::json(&LoginResponse { refresh_token, access_token }))
        }
        None => Err(reject::custom(WrongCredentialsError))
    }
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
