use axum::{Router, routing::get, routing::post, routing::put, routing::delete};
use tokio::net::TcpListener;
mod routing;
use routing::{root, get_user, post_user, get_user_by_id, put_user, delete_user};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/user", get(get_user).post(post_user))
        .route("/user/:id", get(get_user_by_id).put(put_user).delete(delete_user));
    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
