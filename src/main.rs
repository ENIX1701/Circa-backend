use actix_files::{Files, NamedFile};
use actix_web::{App, HttpResponse, HttpServer, web};
use circa_backend::auth;
use circa_backend::auth::delivery::build_magic_link_delivery;
use circa_backend::config::{AppEnvironment, Config};
use circa_backend::db;
use circa_backend::event;
use circa_backend::event::repository::EventRepository;
use circa_backend::event::service::EventService;
use circa_backend::user;
use circa_backend::user::{repository::UserRepository, service::UserService};

async fn spa_index() -> actix_web::Result<NamedFile> {
    Ok(NamedFile::open("./public/index.html")?)
}

async fn api_not_found() -> HttpResponse {
    HttpResponse::NotFound().json(serde_json::json!({
        "error": "API endpoint not found"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let config = Config::init();
    let delivery = build_magic_link_delivery(&config);
    let db_conn = db::establish_connection(&config.database_url)
        .await
        .expect("Failed to connect to the database :c");

    db::reset_from_seed(&db_conn)
        .await
        .expect("Failed to reset database from seed.sql QwQ");
    let reset_db = db_conn.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval_at(
            tokio::time::Instant::now() + tokio::time::Duration::from_secs(60 * 60),
            tokio::time::Duration::from_secs(60 * 60),
        );

        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);

        loop {
            interval.tick().await;

            match db::reset_from_seed(&reset_db).await {
                Ok(()) => log::info!("Database reset from seed.sql"),
                Err(error) => log::error!("Failed to reset database from seed.sql >:c [{error}]"),
            }
        }
    });

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let serve_static_frontend = matches!(config.app_env, AppEnvironment::Production);

    let user_service = web::Data::new(UserService::new(UserRepository::new(db_conn.clone())));
    let event_service = web::Data::new(EventService::new(EventRepository::new(db_conn.clone())));
    let config = web::Data::new(config);
    let delivery = web::Data::new(delivery);
    let db_data = web::Data::new(db_conn);

    println!("Server starting at 0.0.0.0:{port}");

    HttpServer::new(move || {
        App::new()
            .app_data(user_service.clone())
            .app_data(event_service.clone())
            .app_data(config.clone())
            .app_data(delivery.clone())
            .app_data(db_data.clone())
            .service(
                web::scope("/api")
                    .configure(user::routes::config)
                    .configure(auth::routes::config)
                    .configure(event::routes::config)
                    .default_service(web::route().to(api_not_found)),
            )
            .configure(|cfg| {
                if serve_static_frontend {
                    cfg.service(Files::new("/assets", "./public/assets"))
                        .default_service(web::get().to(spa_index));
                }
            })
    })
    .bind(("0.0.0.0", port))?
    .run()
    .await
}
