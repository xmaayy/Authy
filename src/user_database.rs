use crate::error::Error;
use crate::Result as StdResult;
use rusqlite::Connection;
use rusqlite::Result;
//use crate::Result;
use argon2::{self, Config};
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

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
    match initialize_users() {
        Ok(good_conn) => {
            println!("Got connection, inserting user... ");

            // Adding the user to the DB
            // TODO: Check if the user already exists in the database.
            match good_conn.execute(
                "INSERT INTO users (email, password) values (?1, ?2)",
                &[&email, &secure_pass],
            ) {
                Ok(_updated) => println!("Created user entry for {}", email),
                Err(rusqlite::Error::SqliteFailure(err, _)) => {
                    if err.extended_code == 2067 {
                        return Err(Error::UserExistsError);
                        //println!("Bad email (I think) {:?} {:?}", err, newstr);
                    } else {
                        return Err(Error::Unknown{message: err.to_string()});
                    }
                }
                Err(err) => {return Err(Error::Unknown{message: err.to_string()});
},
            }
            let last_id: String = good_conn.last_insert_rowid().to_string();

            //  Creating the first few tokens
            match good_conn.execute(
                "INSERT INTO tokens (user_id, access_token, refresh_token) values (?1, ?2, ?3)",
                &[&last_id, &salt.clone(), &salt.clone()],
            ) {
                Ok(_updated) => println!("Created tokens entry for {}", email),
                Err(err) => println!("update failed: {}", err),
            };
            println!(
                "User {:1} created with:\nuid {:?}\npassword {:?}\nsalt {:?}\nhash {:?}",
                email, last_id, password, salt, secure_pass
            );
            Ok(last_id)
        }
        Err(error) => {
            Err(Error::Unknown{message: error.to_string()})
        }
    }
}

pub fn password_login(email: String, password: String) -> StdResult<String> {
    let db_result = initialize_users();
    let good_conn = match db_result {
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
            println!(
                "PW: {:?}\nArgon String: {:?}\nMatched: {:?}",
                password, user.password, matches
            );
            if matches {
                Ok(String::from(user.id.to_string()))
            } else {
                Err(Error::BadPasswordError)
            }
        }
        Err(err) => {
            match err {
                rusqlite::Error::QueryReturnedNoRows => {return Err(Error::WrongCredentialsError)},
                _ => {return Err(Error::Unknown{message: err.to_string()})},
            }
        }
    }
}
