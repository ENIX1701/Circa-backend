use crate::common::{JWT_SECRET, jwt, test_config};
use actix_web::{App, http::StatusCode, test, web};
use circa_backend::{
    config::AuthDeliveryMode,
    modules::user::entity::{Model, Role, Status},
    user::{
        self,
        models::{CreateUserRequest, UpdateUserRequest, UserRole, UserStatus},
        repository::UserRepository,
        service::UserService,
    },
};
use sea_orm::{DatabaseBackend, MockDatabase};

fn config_data() -> web::Data<circa_backend::config::Config> {
    web::Data::new(test_config(AuthDeliveryMode::Outbox))
}

fn user_model(id: &str, role: Role, status: Status) -> Model {
    Model {
        id: id.to_string(),
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: format!("{id}@example.com"),
        phone: "123".to_string(),
        role,
        status,
        availability_hours: "[]".to_string(),
    }
}

fn app_data(db: sea_orm::DatabaseConnection) -> web::Data<UserService> {
    web::Data::new(UserService::new(UserRepository::new(db)))
}

#[actix_web::test]
async fn authenticated_admin_can_list_create_get_update_and_delete_users() {
    let token = jwt("admin-1", "admin").await;

    let list_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![user_model("user-1", Role::Volunteer, Status::Active)]])
        .into_connection();

    let list_app = test::init_service(
        App::new()
            .app_data(app_data(list_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let list_req = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_eq!(
        test::call_service(&list_app, list_req).await.status(),
        StatusCode::OK
    );

    let create_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![user_model("created", Role::Organizer, Status::Active)]])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let create_app = test::init_service(
        App::new()
            .app_data(app_data(create_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let create_req = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(CreateUserRequest {
            name: "Jane".to_string(),
            surname: "Doe".to_string(),
            email: "jane@example.com".to_string(),
            phone: "456".to_string(),
            role: UserRole::Organizer,
            availability_hours: "[]".to_string(),
        })
        .to_request();
    assert_eq!(
        test::call_service(&create_app, create_req).await.status(),
        StatusCode::OK
    );

    let get_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![user_model("user-1", Role::Volunteer, Status::Active)]])
        .into_connection();

    let get_app = test::init_service(
        App::new()
            .app_data(app_data(get_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let get_req = test::TestRequest::get()
        .uri("/users/user-1")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_eq!(
        test::call_service(&get_app, get_req).await.status(),
        StatusCode::OK
    );

    let update_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![user_model("user-1", Role::Volunteer, Status::Active)],
            vec![user_model("user-1", Role::Staff, Status::Inactive)],
        ])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let update_app = test::init_service(
        App::new()
            .app_data(app_data(update_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let update_req = test::TestRequest::patch()
        .uri("/users/user-1")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(UpdateUserRequest {
            name: Some("Jane".to_string()),
            surname: None,
            email: None,
            phone: None,
            role: Some(UserRole::Staff),
            status: Some(UserStatus::Inactive),
            availability_hours: None,
        })
        .to_request();
    assert_eq!(
        test::call_service(&update_app, update_req).await.status(),
        StatusCode::OK
    );

    let delete_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();

    let delete_app = test::init_service(
        App::new()
            .app_data(app_data(delete_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let delete_req = test::TestRequest::delete()
        .uri("/users/user-1")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    assert_eq!(
        test::call_service(&delete_app, delete_req).await.status(),
        StatusCode::NO_CONTENT
    );
}

#[actix_web::test]
async fn user_routes_reject_missing_and_invalid_tokens() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();
    let app = test::init_service(
        App::new()
            .app_data(app_data(db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let missing = test::TestRequest::get().uri("/users").to_request();
    assert_eq!(
        test::call_service(&app, missing).await.status(),
        StatusCode::UNAUTHORIZED
    );

    let invalid = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", "Bearer no"))
        .to_request();
    assert_eq!(
        test::call_service(&app, invalid).await.status(),
        StatusCode::UNAUTHORIZED
    );

    assert_eq!(JWT_SECRET, "test-secret");
}

#[actix_web::test]
async fn user_routes_return_forbidden_not_found_and_bad_request_errors() {
    let volunteer_token = jwt("volunteer-1", "volunteer").await;
    let admin_token = jwt("admin-1", "admin").await;

    let forbidden_app = test::init_service(
        App::new()
            .app_data(app_data(
                MockDatabase::new(DatabaseBackend::Sqlite).into_connection(),
            ))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let forbidden = test::TestRequest::get()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {volunteer_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&forbidden_app, forbidden).await.status(),
        StatusCode::FORBIDDEN
    );

    let not_found_db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();

    let not_found_app = test::init_service(
        App::new()
            .app_data(app_data(not_found_db))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let not_found = test::TestRequest::get()
        .uri("/users/missing")
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .to_request();
    assert_eq!(
        test::call_service(&not_found_app, not_found).await.status(),
        StatusCode::NOT_FOUND
    );

    let bad_request_app = test::init_service(
        App::new()
            .app_data(app_data(
                MockDatabase::new(DatabaseBackend::Sqlite).into_connection(),
            ))
            .app_data(config_data())
            .configure(user::routes::config),
    )
    .await;

    let invalid_json = test::TestRequest::post()
        .uri("/users")
        .insert_header(("Authorization", format!("Bearer {admin_token}")))
        .insert_header(("Content-Type", "application/json"))
        .set_payload(r#"{"name":"missing required fields"}"#)
        .to_request();
    assert_eq!(
        test::call_service(&bad_request_app, invalid_json)
            .await
            .status(),
        StatusCode::BAD_REQUEST
    );
}
