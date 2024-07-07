#![recursion_limit = "512"]

use axum::{
	routing::{get, post},
	Extension, Form, Router,
	http: {Request, StatusCode},
	response::{Html as ResponseHtml, IntoResponse, Redirect},
};

#[tokio::main]
async fn main() {
	let app = Router::new().route("/", get(|| async { "Hello, World!"}));
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, app).await.unwrap();
}