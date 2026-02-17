use actix_web::{App, HttpServer};
use circa_backend::user;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().configure(user::routes::config))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
