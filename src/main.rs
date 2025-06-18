use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use redb::{Database, ReadableTable, TableDefinition};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use tokio::task;

// User model
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

// REDB table definition
static USER_TABLE: TableDefinition<u32, &str> = TableDefinition::new("user");

// Shared global DB handle using Arc
static DB: Lazy<Arc<Database>> = Lazy::new(|| {
    Arc::new(Database::create("zrqn.redb").expect("Failed to open Redb"))
});

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) if path.starts_with("/user/") => {
            let id_part = path.trim_start_matches("/user/");
            if let Ok(id) = id_part.parse::<u32>() {
                let db = DB.clone();
                let result = task::spawn_blocking(move || {
                    let txn = db.begin_read().unwrap();
                    let table = txn.open_table(USER_TABLE).unwrap();
                    table.get(&id).ok().flatten().map(|val| val.value().to_owned())
                }).await.unwrap();

                if let Some(raw_str) = result {
                    if let Ok(user) = serde_json::from_str::<User>(&raw_str) {
                        let json = serde_json::to_string(&user).unwrap();
                        return Ok(Response::new(Body::from(json)));
                    }
                }
                Ok(not_found())
            } else {
                Ok(bad_request())
            }
        }
        _ => Ok(not_found()),
    }
}

fn not_found() -> Response<Body> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"error":"User not found"}"#))
        .unwrap()
}

fn bad_request() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header("Content-Type", "application/json")
        .body(Body::from(r#"{"error":"Invalid user ID"}"#))
        .unwrap()
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router))
    });

    println!("🚀 Hyper server running at http://{}", addr);
    Server::bind(&addr).serve(make_svc).await.unwrap();
}
