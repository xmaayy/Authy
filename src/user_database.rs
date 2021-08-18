use rusqlite::NO_PARAMS;
use rusqlite::{Connection, Result};
use std::collections::HashMap;


#[derive(Debug)]
struct UserRecord {
    userId: String,
    email: String,
    saltedPass: String,
}

fn initialize() -> Connection() {

    let conn = Connection::open_in_memory()?;

}


fn make_user(email: String, password: String) -> Result<()> {

    /// 
    ///
    ///
    ///


    for (color, catnames) in &cat_colors {
        conn.execute(
            "INSERT INTO cat_colors (name) values (?1)",
            &[&color.to_string()],
        )?;
        let last_id: String = conn.last_insert_rowid().to_string();

        for cat in catnames {
            conn.execute(
                "INSERT INTO cats (name, color_id) values (?1, ?2)",
                &[&cat.to_string(), &last_id],
            )?;
        }
    }
    let mut stmt = conn.prepare(
        "SELECT c.name, cc.name from cats c
         INNER JOIN cat_colors cc
         ON cc.id = c.color_id;",
    )?;

    let cats = stmt.query_map(NO_PARAMS, |row| {
        Ok(Cat {
            name: row.get(0)?,
            color: row.get(1)?,
        })
    })?;

    for cat in cats {
        println!("Found cat {:?}", cat);
    }

    Ok(())
}
