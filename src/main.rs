use axum::{extract::Path, routing::get, Json, Router};
use once_cell::sync::Lazy;
use redb::{Database, ReadableTable, TableDefinition};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::Path as FsPath};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

// Define the table
static USER_TABLE: TableDefinition<u32, &str> = TableDefinition::new("user");

// Lazy load the Redb database
static DB: Lazy<Database> = Lazy::new(|| {
    let db_path = "./zrqn.redb";

    // Ensure parent dir exists
    if let Some(parent) = FsPath::new(db_path).parent() {
        fs::create_dir_all(parent).unwrap();
    }

    Database::create(db_path).expect("Failed to create or open Redb database")
});

async fn get_user(Path(id): Path<u32>) -> Json<Option<User>> {
    let db = &*DB;

    let read_txn = db.begin_read().unwrap();
    let table = read_txn.open_table(USER_TABLE).unwrap();

    let result = table.get(&id).unwrap().map(|entry| {
        let json_str = entry.value();
        serde_json::from_str::<User>(json_str).ok()
    }).flatten();

    Json(result)
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/user/:id", get(get_user));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server listening at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
