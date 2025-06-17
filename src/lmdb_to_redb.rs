use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use std::{fs, time::{SystemTime, UNIX_EPOCH}};

#[derive(Serialize, Deserialize, Debug)]
struct RawUser {
    email: String,
    password: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

static USER_TABLE: TableDefinition<u32, &str> = TableDefinition::new("user");

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load and parse user.json
    let file_data = fs::read_to_string("user.json")?;
    let raw_users: Vec<RawUser> = serde_json::from_str(&file_data)?;

    let db = Database::create("zrqn.redb")?;
    let write_txn = db.begin_write()?;
    {
        let mut table = write_txn.open_table(USER_TABLE)?;

        for (i, raw) in raw_users.into_iter().enumerate() {
            let id = (i + 1) as u32;

            let created_at = SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_secs()
                .to_string();

            let user = User {
                id,
                email: raw.email,
                password: raw.password,
                created_at,
            };

            let json_value = serde_json::to_string(&user)?;
            table.insert(&id, json_value.as_str())?;
        }
    }

    write_txn.commit()?;
    println!("✅ Redb populated with users.");
    Ok(())
}
