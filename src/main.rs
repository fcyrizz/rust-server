use hyper::{Body, Request, Response, Server, Method, StatusCode, header};
use hyper::service::{make_service_fn, service_fn};
use once_cell::sync::OnceCell;
use std::{convert::Infallible, net::SocketAddr, sync::Arc};
use std::collections::HashMap;
use bytes::Bytes;
use parking_lot::RwLock;
use tokio::sync::Semaphore;
use redb::{Database, ReadableTable, TableDefinition};

// Optimized User struct with borrowed deserialization
#[derive(serde::Deserialize)]
struct User<'a> {
    id: u32,
    #[serde(borrow)]
    email: &'a str,
    #[serde(borrow)]
    password: &'a str,
    #[serde(borrow)]
    created_at: &'a str,
}

// Cache type optimized for read-heavy workload
type UserCache = Arc<RwLock<HashMap<u32, Bytes>>>;

// Global cache with OnceCell for initialization control
static CACHE: OnceCell<UserCache> = OnceCell::new();

// Connection limiter to prevent overload
static CONNECTION_SEMAPHORE: OnceCell<Arc<Semaphore>> = OnceCell::new();

async fn router(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // Apply connection limiting
    let _permit = match CONNECTION_SEMAPHORE.get() {
        Some(sem) => sem.acquire().await,
        None => return Ok(service_unavailable()),
    };

    match (req.method(), req.uri().path()) {
        (&Method::GET, path) if path.starts_with("/user/") => {
            let id = path.split('/').nth(2).and_then(|s| s.parse::<u32>().ok());
            
            if let Some(user_id) = id {
                if let Some(cache) = CACHE.get() {
                    let cache = cache.read();
                    if let Some(data) = cache.get(&user_id) {
                        return Ok(Response::builder()
                            .header(header::CONTENT_TYPE, "application/json")
                            .header(header::CACHE_CONTROL, "public, max-age=3600")
                            .body(Body::from(data.clone()))
                            .unwrap());
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
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"error":"User not found"}"#))
        .unwrap()
}

fn bad_request() -> Response<Body> {
    Response::builder()
        .status(StatusCode::BAD_REQUEST)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"error":"Invalid user ID"}"#))
        .unwrap()
}

fn service_unavailable() -> Response<Body> {
    Response::builder()
        .status(StatusCode::SERVICE_UNAVAILABLE)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"error":"Server busy"}"#))
        .unwrap()
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    // Initialize cache
    let cache = Arc::new(RwLock::new(load_cache()));
    CACHE.set(cache).expect("Cache initialization failed");
    
    // Initialize connection limiter (adjust 10000 based on your expected max connections)
    CONNECTION_SEMAPHORE.set(Arc::new(Semaphore::new(10000)))
        .expect("Semaphore initialization failed");

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    
    // Configure hyper server with optimized settings
    let server = Server::bind(&addr)
        .tcp_nodelay(true)
        .tcp_keepalive(None)
        .http1_only(true)
        .http1_title_case_headers(true)
        .http1_preserve_header_case(false);
    
    let make_svc = make_service_fn(|_conn| async {
        Ok::<_, Infallible>(service_fn(router))
    });

    println!("🚀 Optimized Hyper server running at http://{}", addr);
    server.serve(make_svc).await.unwrap();
}

fn load_cache() -> HashMap<u32, Bytes> {
    let db = Database::open("zrqn.redb").expect("Failed to open Redb");
    let txn = db.begin_read().expect("Failed to begin read transaction");
    let table = txn.open_table(TableDefinition::<u32, &str>::new("user"))
        .expect("Failed to open table");
    
    let mut cache = HashMap::new();
    let iter = table.iter().expect("Failed to create iterator");
    
    for item in iter {
        let (id, value) = item.expect("Failed to read item");
        let json_str = value.value();
        
        // Validate JSON structure without full deserialization
        if serde_json::from_str::<User>(json_str).is_ok() {
            cache.insert(id.value(), Bytes::from(json_str.to_owned()));
        }
    }
    
    cache
}
