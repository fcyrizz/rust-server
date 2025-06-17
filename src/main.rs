use hyper::{Body, Request, Response, Server, Method, StatusCode};
use hyper::service::{make_service_fn, service_fn};
use redb::{Database, ReadableTable, TableDefinition};
use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};
use std::{convert::Infallible, net::SocketAddr};

// Define your user struct
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u32,
    email: String,
    password: String,
    created_at: String,
}

// Define REDB table
static USER_TABLE: TableDefinition<u32, &str> = TableDefinition::new("user");

// Lazy-init REDB instance
static DB: Lazy<Database> = Lazy::new(|| {
    Database::create("zrqn.redb").expect("Failed to create/open Redb")
});

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
        (&Method::GET, path) if path.starts_with("/user/") => {
            // Extract user ID from path
            let id_part = path.trim_start_matches("/user/");
            if let Ok(id) = id_part.parse::<u32>() {
                let read_txn = DB.begin_read().unwrap();
                let table = read_txn.open_table(USER_TABLE).unwrap();

                if let Some(val) = table.get(&id).unwrap() {
                    if let Ok(user) = serde_json::from_str::<User>(val.value()) {
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

#[tokio::main]
async fn main() {
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router))
    });

    println!("🚀 Hyper server running at http://{}", addr);
    Server::bind(&addr).serve(make_svc).await.unwrap();
}
