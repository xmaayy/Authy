use auth::{with_auth, Role};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use warp::{reject, reply, Filter, Rejection, Reply};

#[macro_use]
extern crate lazy_static;

mod auth;
mod error;
mod models;
mod user_database;

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;

// This only affects the signing of tokens, so your
// users will get logged out whenever you do a server
// restart. But, it also means that if your secret token
// is ever exposed, you can just fix it by restarting the
// container because lazy statics are created at runtime
lazy_static! {
    static ref JWT_SECRET: Vec<u8> = thread_rng().sample_iter(&Alphanumeric).take(32).collect();
}

#[tokio::main]
async fn main() {
    let create_user_route = warp::path!("create")
        .and(warp::post())
        .and(warp::body::json())
        .and_then(create_user_handler);

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
        .or(create_user_route)
        .recover(error::handle_rejection);
    println!("Starting authentication server on 0.0.0.0:8000");
    warp::serve(routes).run(([0, 0, 0, 0], 8000)).await;
}

fn password_auth() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::body::content_length_limit(1024 * 32)
        .and(warp::body::json())
        .and_then(|body: models::LoginRequest| async move {
            match user_database::password_login(body.clone().email, body.clone().pw) {
                Ok(uid) => Ok(uid),
                Err(_) => Err(reject::custom(error::Error::NoPermissionError)),
            }
        })
}

///
pub async fn validate_user_access(
    uid: String,
    body: models::AccessRequest,
) -> WebResult<impl Reply> {
    // We really dont need to allow much more than this for a small
    // authorization request
    match uid == body.uid {
        true => Ok(uid),
        false => Err(reject::custom(error::Error::NoPermissionError)),
    }
}

pub async fn create_user_handler(body: models::LoginRequest) -> WebResult<impl Reply> {
    let uid: String = match user_database::create_user(body.email, body.pw) {
        Ok(uid) => {
            println!("Created user with UID {:?}", uid);
            uid
        },
        Err(custom_err) => {
            return Err(reject::custom(custom_err));
        },
    };

    Ok(format!("Success, new uid = {:?}", uid))
}

/// Called from the /login endpoint with a username and password in the json payload,
/// this endpoint will create a new access and refresh token for the client.
pub async fn login_handler(uid: String) -> WebResult<impl Reply> {
    println!("UID: {:?}", uid);
    let refresh_token = auth::create_jwt(&uid, &Role::Refresh).map_err(|e| reject::custom(e))?;
    let access_token = auth::create_jwt(&uid, &Role::Access).map_err(|e| reject::custom(e))?;
    Ok(reply::json(&models::LoginResponse {
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
    Ok(reply::json(&models::LoginResponse {
        refresh_token,
        access_token,
    }))
}
