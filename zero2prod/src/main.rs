use axum::{Router, extract::Path, routing::get};

async fn greet(name: Option<Path<String>>) -> String {
    let name = name.map(|Path(n)| n).unwrap_or_else(|| "World".to_string());
    format!("Hello {name}!")
}

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(greet))
        .route("/{name}", get(greet));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("👂 Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
