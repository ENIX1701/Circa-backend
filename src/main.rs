use actix_web::{App, HttpServer, web};
use circa_backend::auth;
use circa_backend::config::Config;
use circa_backend::db;
use circa_backend::user;
use circa_backend::user::{repository::UserRepository, service::UserService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();
    let db_conn = db::establish_connection(&config.database_url)
        .await
        .expect("Failed to connect to the database :c");

    let user_service = web::Data::new(UserService::new(UserRepository::new(db_conn)));
    let jwt_secret = web::Data::new(config.jwt_secret);

    println!("Server starting at 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(user_service.clone())
            .app_data(jwt_secret.clone())
            .configure(user::routes::config)
            .configure(auth::routes::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
