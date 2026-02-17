use dotenvy::dotenv;
use std::env;

pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn init() -> Self {
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env");

        Config { database_url }
    }
}
