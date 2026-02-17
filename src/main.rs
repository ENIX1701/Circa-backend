use actix_web::{App, HttpServer, web};
use circa_backend::config::Config;
use circa_backend::db;
use circa_backend::user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Config::init();
    let db_conn = db::establish_connection(&config.database_url)
        .await
        .expect("Failed to connect to the database :c");

    println!("Server starting at 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_conn.clone()))
            .configure(user::routes::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
