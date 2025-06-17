use lmdb::{Environment, WriteFlags, Transaction};
use serde::{Serialize, Deserialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

fn main() {
    let data = fs::read_to_string("users-compiled.json").expect("Failed to read JSON");
    let users: Vec<User> = serde_json::from_str(&data).expect("Invalid JSON");

    let env = Environment::new()
        .set_max_dbs(10)
        .set_map_size(10 * 1024 * 1024)
        .open(Path::new("./zrqn"))
        .expect("Failed to open LMDB");

    let db = env.open_db(Some("user")).expect("Failed to open DB");

    let mut txn = env.begin_rw_txn().expect("Failed to start RW txn");

    for user in users {
        let key = user.id.to_be_bytes();
        let val = serde_json::to_vec(&user).unwrap();
        txn.put(db, &key, &val, WriteFlags::empty()).unwrap();
        println!("✅ Inserted user {}", user.id);
    }

    txn.commit().expect("Commit failed");
    println!("🎉 All users written to LMDB.");
}
