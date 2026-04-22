use actix_web::{App, HttpServer, web};
use circa_backend::auth;
use circa_backend::auth::delivery::build_magic_link_delivery;
use circa_backend::config::Config;
use circa_backend::db;
use circa_backend::event;
use circa_backend::event::repository::EventRepository;
use circa_backend::event::service::EventService;
use circa_backend::user;
use circa_backend::user::{repository::UserRepository, service::UserService};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Config::init();
    let delivery = build_magic_link_delivery(&config);
    let db_conn = db::establish_connection(&config.database_url)
        .await
        .expect("Failed to connect to the database :c");

    let user_service = web::Data::new(UserService::new(UserRepository::new(db_conn.clone())));
    let event_service = web::Data::new(EventService::new(EventRepository::new(db_conn.clone())));
    let config = web::Data::new(config);
    let delivery = web::Data::new(delivery);
    let db_data = web::Data::new(db_conn);

    println!("Server starting at 0.0.0.0:8080");

    HttpServer::new(move || {
        App::new()
            .app_data(user_service.clone())
            .app_data(event_service.clone())
            .app_data(config.clone())
            .app_data(delivery.clone())
            .app_data(db_data.clone())
            .configure(user::routes::config)
            .configure(auth::routes::config)
            .configure(event::routes::config)
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
