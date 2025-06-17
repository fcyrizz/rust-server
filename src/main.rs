use axum::{
    routing::get,
    Json, Router,
};
use serde::Serialize;
use std::net::SocketAddr;
use tokio::net::TcpListener;

#[derive(Serialize)]
struct Message {
    message: &'static str,
}

async fn root_handler() -> Json<Message> {
    Json(Message {
        message: "Hello from Axum on Termux!",
    })
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(root_handler));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await.unwrap();

    println!("🚀 Listening on http://{}", addr);

    axum::serve(listener, app).await.unwrap();
}
