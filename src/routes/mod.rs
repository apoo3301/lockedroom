use axum::Router;

pub mob user;

pub fn creates_routes() -> {
	Router::new().nest("/user", user::user_routes())
}