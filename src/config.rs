use dotenvy::dotenv;
use std::env;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AppEnvironment {
    Development,
    Production,
}

impl AppEnvironment {
    fn from_env(value: &str) -> Self {
        match value {
            "production" => Self::Production,
            _ => Self::Development,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthDeliveryMode {
    Outbox,
    Smtp, // don't plan on using this, but the implementation is the same, so it may as well be here x3
}

impl AuthDeliveryMode {
    fn from_env(value: &str) -> Self {
        match value {
            "smtp" => Self::Smtp,
            _ => Self::Outbox,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub frontend_url: String,
    pub app_env: AppEnvironment,
    pub auth_delivery_mode: AuthDeliveryMode,
}

impl Config {
    pub fn init() -> Self {
        dotenv().ok();

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env"),
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env"),
            frontend_url: env::var("FRONTEND_URL")
                .unwrap_or_else(|_| "http://localhost:5173".to_string()),
            app_env: AppEnvironment::from_env(
                &env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            ),
            auth_delivery_mode: AuthDeliveryMode::from_env(
                &env::var("AUTH_DELIVERY_MODE").unwrap_or_else(|_| "outbox".to_string()),
            ),
        }
    }

    pub fn test_inbox_enabled(&self) -> bool {
        matches!(self.auth_delivery_mode, AuthDeliveryMode::Outbox)
    }
}
