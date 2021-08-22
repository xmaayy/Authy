use rusqlite::Connection;
use rusqlite::Result;
use std::collections::HashMap;
//use crate::Result;
use argon2::{self, Config};
use rand::{thread_rng, Rng};
use rand::distributions::Alphanumeric;


#[derive(Debug)]
struct UserRecord {
    user_id: String,
    email: String,
    salted_pass: String,
}

pub fn initialize_users() -> Result<Connection> {

    let conn = Connection::open("users.db")?;

    conn.execute(
        "create table if not exists users (
             id integer primary key,
             email text not null,
             password text not null,
             salt text not null
         )",
        [],
    )?;
    //conn.execute(
    //    "create table if not exists tokens (
    //         user_id integer not null references users(id),
    //         access_token text,
    //         refresh_token text,
    //     )",
    //    [],
    //)?;

    Ok(conn)
}


pub fn create_user(email: String, password: String) -> Result<()> {
    
    let pass_bytes = password.clone().into_bytes(); 

    // Generating salt for password storage
    let salt:String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .map(char::from)
        .collect();

    let config = Config::default();
    let secure_pass = argon2::hash_encoded(&password.clone().into_bytes(), &salt.clone().into_bytes(), &config).unwrap();
    let matches =  argon2::verify_encoded(&secure_pass, &password.clone().into_bytes()).unwrap();

    // This isnt how it should be done, its just a placeholder for now until
    // I figure out using an actual DB for users
    let conn = initialize_users();

    match conn {
        Ok(good_conn) => {
            good_conn.execute(
                "INSERT INTO users (email, password, salt) values (?1, ?2, ?3)",
                &[&email, &secure_pass, &salt],
            )?;
            let last_id: String = good_conn.last_insert_rowid().to_string();
            //conn.execute(
            //    "INSERT INTO users (name, color_id) values (?1, ?2)",
            //    &[&cat.to_string(), &last_id],
            //)?;
            println!("User {:1} created with:\nuid {:?}\npassword {:?}\nsalt {:?}\nhash {:?}", email, last_id, password, salt, secure_pass);

        }
        Err(error) => {println!("Error: {}", error);}
    }

    Ok(())
}

fn auth_user() -> Result<()> {
    /*
    let mut stmt = conn.prepare(
        "SELECT c.name, cc.name from cats c
         INNER JOIN cat_colors cc
         ON cc.id = c.color_id;",
    )?;

    let cats = stmt.query_map([] |row| {
        Ok(Cat {
            name: row.get(0)?,
            color: row.get(1)?,
        })
    })?;

    for cat in cats {
        println!("Found cat {:?}", cat);
    }

        */
    Ok(())
}
