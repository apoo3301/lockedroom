use axum::{Router};
use std::net::SocketAddr;
use tokio::signal;
use tower_http::services::ServeDir;

mod routes;

#[tokio::main]

async fn main() {
	let room = routes::create_routes()
		.fallback_service(tower_http::services:ServeDir::new("./static"));

	let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
	println!("the room is running on {}", addr);
	axum::Server::bind(&addr)
		.serve(room.into_make_service())
		.await
		.expect("server failed to run")
		.unwrap();
}