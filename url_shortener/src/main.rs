use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};
use sqlx::{
    migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Error, Executor, Pool, Row, Sqlite,
};

const DEFAULT_SHORT_CODE_LENGTH: usize = 4;

struct AppState {
    base_url: String,
    pool: Pool<Sqlite>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            base_url: self.base_url.clone(),
            pool: self.pool.clone(),
        }
    }
}

#[tokio::main]
async fn main() {
    let db_url = std::env::var("DB_URL").unwrap_or("sqlite://:memory:".to_string());
    let port = std::env::var("PORT").unwrap_or("3000".to_string());
    let base_url = std::env::var("BASE_URL").unwrap_or(format!("http://localhost:{}", port));

    let pool = setup_database(&db_url).await.unwrap();
    let app_state = AppState { base_url, pool };

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

async fn setup_database(db_url: &str) -> Result<Pool<Sqlite>, Error> {
    if !Sqlite::database_exists(db_url).await.unwrap_or(false) {
        Sqlite::create_database(db_url).await?;
    }

    let pool = SqlitePoolOptions::new().connect(db_url).await?;
    pool.execute(
        "CREATE TABLE IF NOT EXISTS short_urls (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            original_url TEXT NOT NULL,
            short_code TEXT NOT NULL UNIQUE,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .await?;
    pool.execute(
        "CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .await?;

    Ok(pool)
}

enum UrlShortenerError {
    NotFound,
    Internal(Error),
}

impl IntoResponse for UrlShortenerError {
    fn into_response(self) -> axum::response::Response {
        match self {
            UrlShortenerError::NotFound => {
                (StatusCode::BAD_REQUEST, "Short URL not found").into_response()
            }
            UrlShortenerError::Internal(error) => {
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
        pool,
    }): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> Result<Json<ShortenResponse>, UrlShortenerError> {
    loop {
        let short_code_length = get_short_code_length(&pool).await?;

        for _ in 0..3 {
            let short_code = generate_short_code(short_code_length);
            let result =
                sqlx::query("INSERT INTO short_urls (original_url, short_code) VALUES (?, ?)")
                    .bind(&payload.original_url)
                    .bind(&short_code)
                    .execute(&pool)
                    .await;

            match result {
                Ok(_) => {
                    let short_url = format!("{}/{}", host, short_code);
                    let response = ShortenResponse {
                        short_code,
                        short_url,
                    };
                    return Ok(Json(response));
                }
                Err(error) => {
                    if let Error::Database(ref database_error) = error {
                        if database_error.is_unique_violation() {
                            continue;
                        }
                    }
                    return Err(UrlShortenerError::Internal(error));
                }
            }
        }

        save_short_code_length(&pool, short_code_length + 1).await?;
    }
}

async fn get_short_code_length(pool: &Pool<Sqlite>) -> Result<usize, UrlShortenerError> {
    let optional_row = sqlx::query("SELECT * FROM settings WHERE key = 'short_code_length'")
        .fetch_optional(pool)
        .await
        .map_err(|error| UrlShortenerError::Internal(error))?;

    match optional_row {
        Some(row) => {
            let value_string = row.get::<String, &str>("value");
            let value = value_string.parse::<usize>().unwrap_or_else(|error| {
                println!("Error parsing short_code_length value: {}", error);
                DEFAULT_SHORT_CODE_LENGTH
            });
            Ok(value)
        }
        None => Ok(DEFAULT_SHORT_CODE_LENGTH),
    }
}

async fn save_short_code_length(
    pool: &Pool<Sqlite>,
    length: usize,
) -> Result<(), UrlShortenerError> {
    let _ = sqlx::query("INSERT INTO settings (key, value) VALUES ('short_code_length', ?) ON CONFLICT DO UPDATE SET value = excluded.value")
    .bind(length.to_string())
        .execute(pool)
        .await.map_err(|error| UrlShortenerError::Internal(error));

    println!("short_code_length has been changed to {}", length);
    Ok(())
}

fn generate_short_code(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}

async fn lookup_handler(
    State(AppState { pool, .. }): State<AppState>,
    Query(short_code): Query<String>,
) -> axum::response::Result<String, UrlShortenerError> {
    let original_url = lookup(&pool, &short_code).await?;
    Ok(original_url)
}

async fn redirect_handler(
    State(AppState { pool, .. }): State<AppState>,
    Path(short_code): Path<String>,
) -> axum::response::Result<Redirect, UrlShortenerError> {
    let original_url = lookup(&pool, &short_code).await?;
    Ok(Redirect::temporary(&original_url))
}

async fn lookup(pool: &Pool<Sqlite>, short_code: &str) -> Result<String, UrlShortenerError> {
    let optional_row = sqlx::query("SELECT * FROM short_urls WHERE short_code = ?")
        .bind(short_code)
        .fetch_optional(pool)
        .await
        .map_err(|error| UrlShortenerError::Internal(error))?;

    match optional_row {
        Some(row) => Ok(row.get::<String, &str>("original_url")),
        None => Err(UrlShortenerError::NotFound),
    }
}
