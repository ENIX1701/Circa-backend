use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub database_url: String,
    pub jwt_secret: String,
    pub frontend_url: String,
}

impl Config {
    pub fn init() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");
        let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env");
        let frontend_url =
            env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:5173".to_string());

        Config {
            database_url,
            jwt_secret,
            frontend_url,
        }
    }
}
