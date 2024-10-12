mod url_shortener;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Pool, Sqlite};
use url_shortener::UrlShortener;

struct AppState {
    base_url: String,
    url_shortener: UrlShortener,
}

impl AppState {
    fn new(base_url: String, url_shortener: UrlShortener) -> Self {
        AppState {
            base_url,
            url_shortener,
        }
    }
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            url_shortener: self.url_shortener.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DB_URL").unwrap_or("sqlite://:memory:".to_string());
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let base_url = std::env::var("BASE_URL").unwrap_or(format!("http://localhost:{}", port));

    let pool = setup_database(&db_url).await.unwrap();

    let url_shortener = UrlShortener::new(pool);
    url_shortener.auto_migrate().await.unwrap();

    let app_state = AppState::new(base_url, url_shortener);

    let app = Router::new()
        .route("/shorten", post(shorten_handler))
        .route("/lookup", get(lookup_handler))
        .route("/:short_code", get(redirect_handler))
        .with_state(app_state);

    let bind_address = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&bind_address).await.unwrap();
    println!("URL Shortener is bound to {}", bind_address);
    axum::serve(listener, app).await.unwrap();
}

async fn setup_database(db_url: &str) -> Result<Pool<Sqlite>, sqlx::Error> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        Sqlite::create_database(db_url).await?;
    }
    SqlitePoolOptions::new().connect(db_url).await
}

impl IntoResponse for url_shortener::Error {
    fn into_response(self) -> axum::response::Response {
        match self {
            url_shortener::Error::NotFound => {
                (StatusCode::NOT_FOUND, "Short URL not found").into_response()
            }
            url_shortener::Error::Internal(error) => {
                (StatusCode::INTERNAL_SERVER_ERROR, error.to_string()).into_response()
            }
        }
    }
}

#[derive(Deserialize)]
struct ShortenRequest {
    original_url: String,
}

#[derive(Serialize)]
struct ShortenResponse {
    short_code: String,
    short_url: String,
}

async fn shorten_handler(
    State(AppState {
        base_url: host,
        url_shortener,
    }): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, url_shortener::Error> {
    let short_code = url_shortener.register(&payload.original_url).await?;

    let short_url = format!("{}/{}", host, short_code);
    let response = ShortenResponse {
        short_code,
        short_url,
    };

    Ok(Json(response))
}

async fn lookup_handler(
    State(AppState { url_shortener, .. }): State<AppState>,
    Query(short_code): Query<String>,
) -> axum::response::Result<String, url_shortener::Error> {
    let original_url = url_shortener.lookup(&short_code).await?;
    Ok(original_url)
}

async fn redirect_handler(
    State(AppState { url_shortener, .. }): State<AppState>,
    Path(short_code): Path<String>,
) -> axum::response::Result<Redirect, url_shortener::Error> {
    let original_url = url_shortener.lookup(&short_code).await?;
    Ok(Redirect::temporary(&original_url))
}
