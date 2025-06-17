use lmdb::{Environment, Database, WriteFlags};
use serde::{Serialize, Deserialize};
use std::{fs, path::Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: u32,
    pub email: String,
    pub password: String,
    pub created_at: String,
}

pub fn setup_lmdb(path: &str) -> (Environment, Database) {
    // Create DB dir if not exists
    if !Path::new(path).exists() {
        fs::create_dir_all(path).unwrap();
    }

    // Setup LMDB env
    let env = Environment::new()
        .set_max_dbs(10)
        .set_map_size(10 * 1024 * 1024) // 10MB for demo
        .open(Path::new(path))
        .unwrap();

    // Open (or create) the "user" DBI
    let db = env.create_db(Some("user"), lmdb::DatabaseFlags::empty()).unwrap();

    // Insert demo data
    let demo_users = vec![
        User { id: 1, email: "alice@example.com".into(), password: "pass1".into(), created_at: "2025-06-17T12:00:00Z".into() },
        User { id: 2, email: "bob@example.com".into(), password: "pass2".into(), created_at: "2025-06-17T12:05:00Z".into() },
        User { id: 3, email: "carol@example.com".into(), password: "pass3".into(), created_at: "2025-06-17T12:10:00Z".into() },
        User { id: 4, email: "dave@example.com".into(), password: "pass4".into(), created_at: "2025-06-17T12:15:00Z".into() },
        User { id: 5, email: "eve@example.com".into(), password: "pass5".into(), created_at: "2025-06-17T12:20:00Z".into() },
    ];

    let mut txn = env.begin_rw_txn().unwrap();
    for user in demo_users {
        let key = user.id.to_be_bytes(); // use id as key
        let value = serde_json::to_vec(&user).unwrap();
        txn.put(db, &key, &value, WriteFlags::empty()).unwrap();
    }
    txn.commit().unwrap();

    (env, db)
}
