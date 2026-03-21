use serde::Deserialize;

/// Application configuration loaded from environment variables.
///
/// All fields are validated at startup via [`Config::from_env`].
/// Missing required variables cause an immediate, descriptive error
/// rather than a runtime panic deep in business logic.
///
/// # Environment Variables
///
/// | Variable | Required | Default | Description |
/// |---|---|---|---|
/// | `DATABASE_URL` | Yes | — | PostgreSQL connection string (PostGIS) |
/// | `REINFOLIB_API_KEY` | No | — | MLIT Real Estate API key |
/// | `PORT` | No | `8000` | HTTP listen port |
/// | `DB_MAX_CONNECTIONS` | No | `20` | Connection pool size |
/// | `RUST_LOG_FORMAT` | No | `pretty` | Log format: `json` or `pretty` |
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// PostgreSQL connection string (PostGIS-enabled).
    pub database_url: String,

    /// Optional MLIT reinfolib API key.
    #[serde(default)]
    pub reinfolib_api_key: Option<String>,

    /// HTTP listen port.
    #[serde(default = "default_port")]
    pub port: u16,

    /// Maximum number of database connections in the pool.
    #[serde(default = "default_db_max_connections")]
    pub db_max_connections: u32,

    /// Log output format: `json` for structured production logs, anything else for pretty dev output.
    #[serde(default)]
    pub rust_log_format: Option<String>,
}

fn default_port() -> u16 {
    8000
}

fn default_db_max_connections() -> u32 {
    20
}

impl Config {
    /// Load configuration from environment variables.
    ///
    /// Panics with a descriptive message if required variables are missing
    /// or values cannot be parsed (e.g., non-numeric `PORT`).
    /// This is intentional — configuration errors must be caught at startup.
    pub fn from_env() -> Self {
        envy::from_env::<Self>().expect("Failed to load configuration from environment variables")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_values_are_sensible() {
        assert_eq!(default_port(), 8000);
        assert_eq!(default_db_max_connections(), 20);
    }

    #[test]
    fn rust_log_format_none_means_pretty() {
        let config = Config {
            database_url: "postgres://localhost/test".into(),
            reinfolib_api_key: None,
            port: 8000,
            db_max_connections: 20,
            rust_log_format: None,
        };
        assert!(config.rust_log_format.is_none());
    }

    #[test]
    fn rust_log_format_json_is_preserved() {
        let config = Config {
            database_url: "postgres://localhost/test".into(),
            reinfolib_api_key: None,
            port: 8000,
            db_max_connections: 20,
            rust_log_format: Some("json".into()),
        };
        assert_eq!(config.rust_log_format.as_deref(), Some("json"));
    }
}
