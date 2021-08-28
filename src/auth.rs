use crate::{error::Error, Result, WebResult};
use chrono::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::fmt;
use warp::{
    filters::header::headers_cloned,
    http::header::{HeaderMap, HeaderValue, AUTHORIZATION},
    reject, Filter, Rejection,
};

const BEARER: &str = "Bearer ";
/// The idea behind a refresh and a login token is that if
/// we have a very short expiry time on the login token then
/// even if an attacker somehow exfiltrates it, it wont be
/// useful for long. Furthermore, we can hide the refresh
/// token in an HttpCookie so its not as prone to being
/// grabbed.
#[derive(Clone, PartialEq)]
pub enum Role {
    Refresh, // This role is used only the get a new login token
    Access,  // This role is used to access the site normally
    Unauth,
}

impl Role {
    pub fn from_str(role: &str) -> Role {
        match role {
            "Refresh" => Role::Refresh,
            "Access" => Role::Access,
            _ => Role::Unauth,
        }
    }
}

impl fmt::Display for Role {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Role::Access => write!(f, "Access"),
            Role::Refresh => write!(f, "Refresh"),
            Role::Unauth => write!(f, "Invalid"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct Claims {
    sub: String,
    role: String,
    exp: usize,
}

pub fn with_auth(role: Role) -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    // A simple function that calls wraps a warp filter and calls
    // the authentication function with a specific role. Not totally
    // sure how we can pull in the warp headers here, but I guess its
    // because this is a 'Filter' so it has access to a lot of stuff
    // from warp included by default.
    headers_cloned()
        .map(move |headers: HeaderMap<HeaderValue>| (role.clone(), headers))
        .and_then(authorize)
}

pub fn create_jwt(uid: &str, role: &Role) -> Result<String> {
    let expiration: i64;
    if role == &Role::Refresh {
        // Refresh tokens have limited permissions but have a 14
        // day window in which they're active. As soon as one is used,
        // though, it becomes invalid along with the previous login token
        expiration = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(60 * 60 * 24 * 14))
            .expect("valid timestamp")
            .timestamp();
    } else {
        // Any other token (namely login/access tokens) have approx
        // 30 minutes of being valid. This gives a user enough time
        // to get through a single sessions without their client
        // having to reauthentiate, but they will need to request
        // a new access token using their refresh token the next time
        // they visit.
        expiration = Utc::now()
            .checked_add_signed(chrono::Duration::seconds(60 * 30))
            .expect("valid timestamp")
            .timestamp();
    }

    let claims = Claims {
        sub: uid.to_owned(),
        role: role.to_string(),
        exp: expiration as usize,
    };

    let header = Header::new(Algorithm::HS512);
    encode(&header, &claims, &EncodingKey::from_secret(&super::JWT_SECRET))
        .map_err(|_| Error::JWTTokenCreationError)
}

async fn authorize((role, headers): (Role, HeaderMap<HeaderValue>)) -> WebResult<String> {
    match jwt_from_header(&headers) {
        Ok(jwt) => {
            let decoded = decode::<Claims>(
                &jwt,
                &DecodingKey::from_secret(&super::JWT_SECRET),
                // This is really important. We want to make sure that we're not acceping an
                // alternative validation algorithm from within the JWT because then we open
                // ourselves up to the alg::None scenario
                &Validation::new(Algorithm::HS512),
            )
            .map_err(|_| reject::custom(Error::JWTTokenError))?;

            if role == Role::Refresh && Role::from_str(&decoded.claims.role) != Role::Refresh {
                return Err(reject::custom(Error::NoPermissionError));
            }

            Ok(decoded.claims.sub)
        }
        Err(e) => return Err(reject::custom(e)),
    }
}

fn jwt_from_header(headers: &HeaderMap<HeaderValue>) -> Result<String> {
    let header = match headers.get(AUTHORIZATION) {
        Some(v) => v,
        None => return Err(Error::NoAuthHeaderError),
    };
    let auth_header = match std::str::from_utf8(header.as_bytes()) {
        Ok(v) => v,
        Err(_) => return Err(Error::NoAuthHeaderError),
    };
    if !auth_header.starts_with(BEARER) {
        return Err(Error::InvalidAuthHeaderError);
    }
    Ok(auth_header.trim_start_matches(BEARER).to_owned())
}
