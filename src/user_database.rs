use crate::error::Error;
use crate::Result as StdResult;
use argon2::{self, Config};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rusqlite::Connection;
use rusqlite::Result;
use std::fs;

#[derive(Debug, Clone)]
struct UserRecord {
    id: u64,
    email: String,
    password: String,
    access_token: String,
    refresh_token: String,
}

pub fn initialize_users() -> StdResult<Connection> {
    // Creating the folder for the database, make sure you add this as a volume
    // in docker if you want the database to persist
    fs::create_dir_all("database").expect("Couldnt create database folder");
    // creating the database in the folder we just made
    let conn: Connection = Connection::open("database/users.db").expect("Failed to open database file");

    // This is a really ugly statement because of how I did the error handling,
    // but really all we're doing here is creating the user table
    conn.execute(
        "create table if not exists users (
             id integer primary key,
             email text not null unique,
             password text not null
         )",
        [],
    ).expect("Couldnt create user table");

    // Same here but for tokens
    conn.execute(
        "create table if not exists tokens (
             user_id integer not null references users(id),
             access_token text,
             refresh_token text
         )",
        [],
    ).expect("Couldnt create token table");

    Ok(conn)
}

/// The create_user function is called when someone makes a request to the /create
/// endpoint. It is the most costly of all the functions involved in user management
/// because it has 3 database accesses and a heavy password generation operation. There
/// might be a way to optimize the number of database accesses, but the this function
/// will likely always be the slowest.
pub fn create_user(email: String, password: String) -> StdResult<String> {
    // Generating salt for password storage
    let salt: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let config = Config::default();
    let secure_pass = argon2::hash_encoded(
        &password.clone().into_bytes(),
        &salt.clone().into_bytes(),
        &config,
    )
    .unwrap();

    // Test that we got a good connection from the database. This may
    // or not be re-usable once I move to diesel...
    let good_conn = match initialize_users() {
        Ok(good_conn) => good_conn,
        Err(_error) => {
            return Err(Error::DatabaseOperationError);
        }
    };
    // Adding the user to the DB
    match good_conn.execute(
        "INSERT INTO users (email, password) values (?1, ?2)",
        &[&salt.clone(), &secure_pass],
    ) {
        Ok(_) => println!("Created user entry for {}", email),
        Err(rusqlite::Error::SqliteFailure(err, _)) => {
            if err.extended_code == 2067 {
                return Err(Error::UserExistsError);
            } else {
                return Err(Error::Unknown {
                    message: err.to_string(),
                });
            }
        }
        Err(err) => {
            return Err(Error::Unknown {
                message: err.to_string(),
            });
        }
    }
    let last_id: String = good_conn.last_insert_rowid().to_string();
    Ok(last_id)
}

pub fn password_login(email: String, password: String) -> StdResult<String> {
    let good_conn = match initialize_users() {
        Ok(good_conn) => good_conn,
        Err(_error) => {
            return Err(Error::DatabaseOperationError);
        }
    };

    let res: Result<UserRecord, rusqlite::Error> = good_conn.query_row_and_then(
        "SELECT u.id, u.email, u.password
             FROM users u
             WHERE u.email = (?);",
        [&email],
        |row| {
            Ok(UserRecord {
                id: row.get(0)?,
                email: row.get(1)?,
                password: row.get(2)?,
                access_token: String::from(""),
                refresh_token: String::from(""),
            })
        },
    );

    match res {
        Ok(user) => {
            let matches =
                argon2::verify_encoded(&user.password, &password.clone().into_bytes()).unwrap();
            if matches {
                Ok(String::from(user.id.to_string()))
            } else {
                Err(Error::BadPasswordError)
            }
        }
        Err(err) => match err {
            rusqlite::Error::QueryReturnedNoRows => return Err(Error::WrongCredentialsError),
            _ => {
                return Err(Error::Unknown {
                    message: err.to_string(),
                })
            }
        },
    }
}
