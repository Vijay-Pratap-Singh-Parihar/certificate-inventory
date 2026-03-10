use std::env;

/// Application configuration (TOML + env); loaded at startup.
#[derive(Debug, Clone)]
pub struct AppConfig {
    port: u16,
    database_url: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        dotenv::dotenv().ok();
        let port = env::var("APP_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .unwrap_or(8080);
        let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| {
            "postgres://postgres:postgres@localhost:5432/certificate_db".to_string()
        });
        Ok(Self { port, database_url })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn database_url(&self) -> &str {
        &self.database_url
    }
}
