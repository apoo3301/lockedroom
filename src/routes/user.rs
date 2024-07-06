use axum::{routing::get, Router};

pub fn user_routes () -> Router {
	Router::new()
		.route("/", get(get_user))
		.route("/:id", get(get_user_by_id))
		.route("/", post(post_user))
		.route("/:id", put(put_user))
		.route("/:id", delete(delete_user))
}

async fn get_user() -> &'static str {
	"// get all users"
}

async fn get_user_by_id() -> &'static str {
	"// get user by id"
}

async fn post_user() -> &'static str {
	"// create user"
}

async fn put_user() -> &'static str {
	"// update user"
}

async fn delete_user() -> &'static str {
	"// delete user"
}