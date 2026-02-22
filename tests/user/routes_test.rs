use actix_web::{App, http::StatusCode, test, web};
use circa_backend::auth::service::generate_jwt;
use circa_backend::modules::user::entity::{Model, Role, Status};
use circa_backend::user;
use circa_backend::user::models::{CreateUserRequest, UpdateUserRequest, UserRole};
use circa_backend::user::repository::UserRepository;
use circa_backend::user::service::UserService;
use sea_orm::{DatabaseBackend, MockDatabase};

const JWT_SECRET: &str = "test_secret";

fn make_jwt_secret() -> web::Data<String> {
    web::Data::new(JWT_SECRET.to_string())
}

async fn make_admin_token() -> String {
    let resp = generate_jwt("admin@example.com", "admin", JWT_SECRET)
        .await
        .unwrap();
    resp.token
}

fn setup_app_data_with_list() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![Model {
            id: "1".to_string(),
            name: "John".to_string(),
            surname: "Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "123".to_string(),
            role: Role::Admin,
            status: Status::Active,
        }]])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

fn setup_app_data_for_create() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![Model {
            id: "1".to_string(),
            name: "John".to_string(),
            surname: "Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "123".to_string(),
            role: Role::Organizer,
            status: Status::Active,
        }]])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    web::Data::new(UserService::new(UserRepository::new(db)))
}

#[actix_web::test]
async fn test_get_users_route() {
    let token = make_admin_token().await;

    let app = test::init_service(
        App::new()
            .app_data(setup_app_data_with_list())
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_users_route_unauthorized() {
    let app = test::init_service(
        App::new()
            .app_data(setup_app_data_with_list())
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
async fn test_create_user_route() {
    let token = make_admin_token().await;

    let app = test::init_service(
        App::new()
            .app_data(setup_app_data_for_create())
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req_body = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "john@example.com".to_string(),
        phone: "123".to_string(),
        role: UserRole::Organizer,
    };

    let req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_user_by_id_route() {
    let token = make_admin_token().await;

    let app = test::init_service(
        App::new()
            .app_data(setup_app_data_with_list())
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/users/1")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_update_user_route() {
    let token = make_admin_token().await;

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![Model {
                id: "1".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Admin,
                status: Status::Active,
            }],
            vec![Model {
                id: "1".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Admin,
                status: Status::Active,
            }],
        ])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let app_data = web::Data::new(UserService::new(UserRepository::new(db)));

    let app = test::init_service(
        App::new()
            .app_data(app_data)
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req_body = UpdateUserRequest {
        name: Some("Jane".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
    };

    let req = test::TestRequest::patch()
        .uri("/users/1")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_delete_user_route() {
    let token = make_admin_token().await;

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let app_data = web::Data::new(UserService::new(UserRepository::new(db)));

    let app = test::init_service(
        App::new()
            .app_data(app_data)
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::delete()
        .uri("/users/1")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_delete_user_route_unauthorized() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let app_data = web::Data::new(UserService::new(UserRepository::new(db)));

    let app = test::init_service(
        App::new()
            .app_data(app_data)
            .app_data(make_jwt_secret())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::delete().uri("/users/1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}
