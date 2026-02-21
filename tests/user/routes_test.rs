use actix_web::{App, http::StatusCode, test, web};
use circa_backend::modules::user::entity::{Model, Role, Status};
use circa_backend::user;
use circa_backend::user::models::{CreateUserRequest, UpdateUserRequest, UserRole};
use circa_backend::user::repository::UserRepository;
use circa_backend::user::service::UserService;
use sea_orm::{DatabaseBackend, MockDatabase};

fn setup_app_data() -> web::Data<UserService> {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([[Model {
            id: "1".to_string(),
            name: "John".to_string(),
            surname: "Doe".to_string(),
            email: "john@example.com".to_string(),
            phone: "123".to_string(),
            role: Role::EventDirector,
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
    let app = test::init_service(
        App::new()
            .app_data(setup_app_data())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::get().uri("/users").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_create_user_route() {
    let app = test::init_service(
        App::new()
            .app_data(setup_app_data())
            .configure(user::routes::config),
    )
    .await;

    let req_body = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "john@example.com".to_string(),
        phone: "123".to_string(),
        role: UserRole::BoothOwner,
    };

    let req = test::TestRequest::post()
        .uri("/users")
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_get_user_by_id_route() {
    let app = test::init_service(
        App::new()
            .app_data(setup_app_data())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::get().uri("/users/1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_update_user_route() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![Model {
                id: "1".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::EventDirector,
                status: Status::Active,
            }],
            vec![Model {
                id: "1".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::EventDirector,
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
        .set_json(&req_body)
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}

#[actix_web::test]
async fn test_delete_user_route() {
    let app = test::init_service(
        App::new()
            .app_data(setup_app_data())
            .configure(user::routes::config),
    )
    .await;

    let req = test::TestRequest::delete().uri("/users/1").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::OK);
}
