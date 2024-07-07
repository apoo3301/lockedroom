#![recursion_limit = "512"]

use axum::{
	routing::{get, post},
	Extension, Form, Router,
	http::{Request, StatusCode},
	response::{Html as ResponseHtml, IntoResponse, Redirect},
};

use database:: { init_db, User, Ban, Mod, Post };

mod database;

#[tokio::main]
async fn main() {

	let database_conn = Arc::new(Mutex::new(init_db().unwrap()));
	{
		let conn = init_db().lock().await;

	}

	let app = Router::new()
		.route("/", get(|| async { "Hello, World!"}));
	
	
	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
	axum::serve(listener, app).await.unwrap();
	println!("server up on: port 3000");
}