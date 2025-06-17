use axum::{extract::Path, routing::get, Json, Router};
use lmdb::{Environment, Database, Transaction};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, path::Path as FsPath};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

// Use Lazy statics (safe and concurrent)
static LMDB: Lazy<(Environment, Database)> = Lazy::new(|| {
    let env = Environment::new()
        .set_max_dbs(10)
        .set_map_size(10 * 1024 * 1024)
        .open(FsPath::new("./zrqn"))
        .expect("Failed to open LMDB");

    let db = env.open_db(Some("user")).expect("Failed to open DB");

    (env, db)
});

async fn get_user(Path(id): Path<u32>) -> Json<Option<User>> {
    let (env, db) = &*LMDB;
    let txn = env.begin_ro_txn().unwrap();

    let result = match txn.get(*db, &id.to_be_bytes()) {
        Ok(bytes) => serde_json::from_slice(bytes).ok(),
        Err(_) => None,
    };

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
