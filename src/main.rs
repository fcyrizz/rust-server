use axum::{extract::{Path, Extension}, routing::get, Json, Router};
use lmdb::{Environment, Database, WriteFlags, Transaction};
use serde::{Deserialize, Serialize};
use std::{fs, net::SocketAddr, path::Path as FsPath, sync::Arc};
use tokio::net::TcpListener;

#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

type DbHandle = Arc<(Environment, Database)>;

fn setup_lmdb(path: &str) -> (Environment, Database) {
    if !FsPath::new(path).exists() {
        fs::create_dir_all(path).unwrap();
    }

    let env = Environment::new()
        .set_max_dbs(10)
        .set_map_size(10 * 1024 * 1024)
        .open(FsPath::new(path))
        .unwrap();

    let db = env
        .create_db(Some("user"), lmdb::DatabaseFlags::empty())
        .unwrap();

    // Demo data
    let demo_users = vec![
        User { id: 1, email: "alice@example.com".into(), password: "pass1".into(), created_at: "2025-06-17T12:00:00Z".into() },
        User { id: 2, email: "bob@example.com".into(), password: "pass2".into(), created_at: "2025-06-17T12:05:00Z".into() },
        User { id: 3, email: "carol@example.com".into(), password: "pass3".into(), created_at: "2025-06-17T12:10:00Z".into() },
        User { id: 4, email: "dave@example.com".into(), password: "pass4".into(), created_at: "2025-06-17T12:15:00Z".into() },
        User { id: 5, email: "eve@example.com".into(), password: "pass5".into(), created_at: "2025-06-17T12:20:00Z".into() },
    ];

    let mut txn = env.begin_rw_txn().unwrap();
    for user in demo_users {
        let key = user.id.to_be_bytes();
        let value = serde_json::to_vec(&user).unwrap();
        txn.put(db, &key, &value, WriteFlags::empty()).unwrap();
    }
    txn.commit().unwrap();

    (env, db)
}

async fn get_user(
    Path(id): Path<u32>,
    Extension(handle): Extension<DbHandle>,
) -> Json<Option<User>> {
    let (env, db) = &*handle;
    let txn = env.begin_ro_txn().unwrap();
    let user = match txn.get(*db, &id.to_be_bytes()) {
        Ok(bytes) => serde_json::from_slice::<User>(bytes).ok(),
        Err(_) => None,
    };
    Json(user)
}

#[tokio::main]
async fn main() {
    let (env, db) = setup_lmdb("./zrqn");
    let handle: DbHandle = Arc::new((env, db));

    let app = Router::new()
        .route("/user/:id", get(get_user))
        .layer(Extension(handle));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();
    println!("🚀 Running at http://{}", addr);
    axum::serve(listener, app).await.unwrap();
}
