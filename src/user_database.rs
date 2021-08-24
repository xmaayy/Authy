use crate::error::Error;
use crate::Result as StdResult;
use rusqlite::Connection;
use rusqlite::Result;
use std::collections::HashMap;
//use crate::Result;
use argon2::{self, Config};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use warp::{reject, Filter, Rejection};

#[derive(Debug, Clone)]
struct UserRecord {
    id: u64,
    email: String,
    password: String,
    access_token: String,
    refresh_token: String,
}

pub fn initialize_users() -> Result<Connection> {
    let conn = Connection::open("users.db")?;

    conn.execute(
        "create table if not exists users (
             id integer primary key,
             email text not null unique,
             password text not null
         )",
        [],
    )?;
    conn.execute(
        "create table if not exists tokens (
             user_id integer not null references users(id),
             access_token text,
             refresh_token text
         )",
        [],
    )?;

    Ok(conn)
}

/// The create_user function is called when someone makes a request to the /create
/// endpoint. It is the most costly of all the functions involved in user management
/// because it has 3 database accesses and a heavy password generation operation. There
/// might be a way to optimize the number of database accesses, but the this function
/// will likely always be the slowest.
pub fn create_user(email: String, password: String) -> StdResult<String> {
    let pass_bytes = password.clone().into_bytes();

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
    let matches = argon2::verify_encoded(&secure_pass, &password.clone().into_bytes()).unwrap();

    // This isnt how it should be done, its just a placeholder for now until
    // I figure out using an actual DB for users
    let conn = initialize_users();

    // Test that we got a good connection from the database. This may
    // or not be re-usable once I move to diesel...
    match conn {
        Ok(good_conn) => {
            println!("Got connection, inserting user... ");

            // Adding the user to the DB
            // TODO: Check if the user already exists in the database.
            match good_conn.execute(
                "INSERT INTO users (email, password) values (?1, ?2)",
                &[&email, &secure_pass],
            ) {
                Ok(updated) => println!("Created user entry for {}", email),
                Err(rusqlite::Error::SqliteFailure(err, newstr)) => {
                    if err.extended_code == 2067 {
                        return Err(Error::UserExistsError);
                        //println!("Bad email (I think) {:?} {:?}", err, newstr);
                    } else {
                        println!("Haven't dealt with this error yet {:?} {:?}", err, newstr);
                    }
                }
                Err(err) => println!("update failed: {:?}", err),
            }
            let last_id: String = good_conn.last_insert_rowid().to_string();

            //  Creating the first few tokens
            match good_conn.execute(
                "INSERT INTO tokens (user_id, access_token, refresh_token) values (?1, ?2, ?3)",
                &[&last_id, &salt.clone(), &salt.clone()],
            ) {
                Ok(updated) => println!("Created tokens entry for {}", email),
                Err(err) => println!("update failed: {}", err),
            };
            println!(
                "User {:1} created with:\nuid {:?}\npassword {:?}\nsalt {:?}\nhash {:?}",
                email, last_id, password, salt, secure_pass
            );
            Ok(last_id)
        }
        Err(error) => {
            println!("Error: {}", error);
            Err(Error::Unknown)
        }
    }
}

pub fn password_login(email: String, password: String) -> Result<String> {
    // TODO Shift all of the error reporting from rusqlite errors
    // to normal ones (too lazy rn)
    let conn = initialize_users();
    let mut matched_user: Option<UserRecord> = None;
    match conn {
        Ok(good_conn) => {
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
                    // Handling a user being matched
                    println!("Record: {:?}", user);
                    matched_user = Some(user.clone());
                }
                Err(err) => {
                    // Handling the million cases where a user doesn't get matched
                    return Err(err);
                }
            }

        }
        Err(error) => {
            println!("Error: {}", error);
            return Err(error);
        }
    }

    match matched_user {
        Some(user) => {
            let matches = argon2::verify_encoded(&user.password, &password.clone().into_bytes()).unwrap();
            println!("PW: {:?}\nArgon String: {:?}\nMatched: {:?}", password, user.password, matches);
            if matches {
                Ok(String::from(user.id.to_string()))
            } else {
                Ok(String::from("Bad password"))
            }
        },
        None => Ok(String::from("Dunno")),
    }
}


pub fn list_users() -> Result<()> {
    let conn = initialize_users();

    match conn {
        Ok(good_conn) => {
            let mut stmt = good_conn.prepare(
                "SELECT u.id, u.email, u.password, t.access_token, t.refresh_token FROM tokens t
                 INNER JOIN users u
                 ON u.id = t.user_id;",
            )?;

            let res = stmt.query_map([], |row| {
                Ok(UserRecord {
                    id: row.get(0)?,
                    email: row.get(1)?,
                    password: row.get(2)?,
                    access_token: row.get(3)?,
                    refresh_token: row.get(4)?,
                })
            });

            match res {
                Ok(users) => {
                    for user in users {
                        println!("Record: {:?}", user.unwrap())
                    }
                }
                Err(err) => println!("update failed: {}", err),
            }
        }
        Err(error) => {
            println!("Error: {}", error);
        }
    }
    Ok(())
}
