use rand::{distributions::Alphanumeric, Rng};
use sqlx::{Executor, Pool, Row, Sqlite};

const DEFAULT_SHORT_CODE_LENGTH: usize = 4;

pub enum Error {
    NotFound,
    Internal(sqlx::Error),
}

pub struct UrlShortener {
    pool: Pool<Sqlite>,
}

impl UrlShortener {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        UrlShortener { pool }
    }

    pub async fn auto_migrate(&self) -> Result<(), sqlx::Error> {
        self.pool
            .execute(
                "CREATE TABLE IF NOT EXISTS short_urls (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                original_url TEXT NOT NULL,
                short_code TEXT NOT NULL UNIQUE,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            )
            .await?;

        self.pool
            .execute(
                "CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )",
            )
            .await?;

        Ok(())
    }

    pub async fn register(&self, original_url: &str) -> Result<String, Error> {
        loop {
            let short_code_length = self.get_short_code_length().await?;

            for _ in 0..3 {
                let short_code = generate_short_code(short_code_length);
                let result =
                    sqlx::query("INSERT INTO short_urls (original_url, short_code) VALUES (?, ?)")
                        .bind(&original_url)
                        .bind(&short_code)
                        .execute(&self.pool)
                        .await;

                match result {
                    Ok(_) => {
                        return Ok(short_code);
                    }
                    Err(error) => {
                        if let sqlx::Error::Database(ref database_error) = error {
                            if database_error.is_unique_violation() {
                                continue;
                            }
                        }
                        return Err(Error::Internal(error));
                    }
                }
            }

            self.save_short_code_length(short_code_length + 1).await?;
        }
    }

    async fn get_short_code_length(&self) -> Result<usize, Error> {
        let optional_row = sqlx::query("SELECT * FROM settings WHERE key = 'short_code_length'")
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| Error::Internal(error))?;

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

    async fn save_short_code_length(&self, length: usize) -> Result<(), Error> {
        let _ = sqlx::query("INSERT INTO settings (key, value) VALUES ('short_code_length', ?) ON CONFLICT DO UPDATE SET value = excluded.value")
            .bind(length.to_string())
            .execute(&self.pool)
            .await.map_err(|error| Error::Internal(error));

        println!("short_code_length has been changed to {}", length);
        Ok(())
    }

    pub async fn lookup(&self, short_code: &str) -> Result<String, Error> {
        let optional_row = sqlx::query("SELECT * FROM short_urls WHERE short_code = ?")
            .bind(short_code)
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| Error::Internal(error))?;

        match optional_row {
            Some(row) => Ok(row.get::<String, &str>("original_url")),
            None => Err(Error::NotFound),
        }
    }
}

impl Clone for UrlShortener {
    fn clone(&self) -> Self {
        Self {
            pool: self.pool.clone(),
        }
    }
}

fn generate_short_code(length: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(length)
        .map(char::from)
        .collect()
}
