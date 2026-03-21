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
/// | `ALLOWED_ORIGINS` | No | — | Comma-separated allowed CORS origins (e.g. `https://app.example.com,https://staging.example.com`). If unset, all origins are allowed (dev mode). |
/// | `RATE_LIMIT_RPM` | No | `120` | Rate limit: requests per minute per IP |
/// | `RATE_LIMIT_BURST` | No | `20` | Rate limit: burst capacity per IP |
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

    /// Comma-separated list of allowed CORS origins.
    /// When set, only these origins can make cross-origin requests.
    /// When unset (`None`), all origins are allowed (development mode).
    #[serde(default)]
    pub allowed_origins: Option<String>,

    /// Maximum sustained request rate per IP (requests per minute).
    #[serde(default = "default_rate_limit_rpm")]
    pub rate_limit_rpm: u64,

    /// Instantaneous burst capacity per IP above the sustained rate.
    #[serde(default = "default_rate_limit_burst")]
    pub rate_limit_burst: u32,
}

fn default_port() -> u16 {
    8000
}

fn default_db_max_connections() -> u32 {
    20
}

fn default_rate_limit_rpm() -> u64 {
    120
}

fn default_rate_limit_burst() -> u32 {
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

    /// Parse `allowed_origins` into a list of origin strings.
    ///
    /// Returns `None` when origins are unset (permissive dev mode).
    /// Returns `Some(vec)` with trimmed, non-empty origin strings for production.
    pub fn parsed_origins(&self) -> Option<Vec<String>> {
        self.allowed_origins.as_ref().map(|origins| {
            origins
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> Config {
        Config {
            database_url: "postgres://localhost/test".into(),
            reinfolib_api_key: None,
            port: 8000,
            db_max_connections: 20,
            rust_log_format: None,
            allowed_origins: None,
            rate_limit_rpm: 120,
            rate_limit_burst: 20,
        }
    }

    #[test]
    fn default_values_are_sensible() {
        assert_eq!(default_port(), 8000);
        assert_eq!(default_db_max_connections(), 20);
        assert_eq!(default_rate_limit_rpm(), 120);
        assert_eq!(default_rate_limit_burst(), 20);
    }

    #[test]
    fn rust_log_format_none_means_pretty() {
        let config = test_config();
        assert!(config.rust_log_format.is_none());
    }

    #[test]
    fn rust_log_format_json_is_preserved() {
        let mut config = test_config();
        config.rust_log_format = Some("json".into());
        assert_eq!(config.rust_log_format.as_deref(), Some("json"));
    }

    #[test]
    fn parsed_origins_none_when_unset() {
        let config = test_config();
        assert!(config.parsed_origins().is_none());
    }

    #[test]
    fn parsed_origins_splits_comma_separated() {
        let mut config = test_config();
        config.allowed_origins =
            Some("https://app.example.com, https://staging.example.com".into());
        let origins = config.parsed_origins().unwrap();
        assert_eq!(origins.len(), 2);
        assert_eq!(origins[0], "https://app.example.com");
        assert_eq!(origins[1], "https://staging.example.com");
    }

    #[test]
    fn parsed_origins_filters_empty_entries() {
        let mut config = test_config();
        config.allowed_origins = Some("https://app.example.com,,, ".into());
        let origins = config.parsed_origins().unwrap();
        assert_eq!(origins.len(), 1);
        assert_eq!(origins[0], "https://app.example.com");
    }
}
