use circa_backend::auth::models::Claims;
use circa_backend::user::{
    entity::{Model, Role, Status},
    models::{CreateUserRequest, UpdateUserRequest, UserRole},
    repository::UserRepository,
    service::UserService,
};
use sea_orm::{DatabaseBackend, MockDatabase};

fn make_claims(sub: &str, role: &str) -> Claims {
    Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        exp: 9999999999,
    }
}

fn setup_mock_db_with_user() -> sea_orm::DatabaseConnection {
    MockDatabase::new(DatabaseBackend::Sqlite)
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
        .into_connection()
}

// ── get_users ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_users_success() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![
            Model {
                id: "1".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Admin,
                status: Status::Active,
            },
            Model {
                id: "2".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "jane@example.com".to_string(),
                phone: "456".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            },
        ]])
        .into_connection();

    let service = UserService::new(UserRepository::new(db));
    let result = service.get_users().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_users_empty() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();

    let service = UserService::new(UserRepository::new(db));
    let result = service.get_users().await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap().len(), 0);
}

// ── get_user ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_user_success() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user("1").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "1");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user("999").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Not found: User not found");
}

// ── get_user_by_email ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_user_by_email_success() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user_by_email("john@example.com").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().email, "john@example.com");
}

#[tokio::test]
async fn test_get_user_by_email_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user_by_email("nobody@example.com").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Not found: User not found");
}

// ── create_user ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_create_user_success() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));

    let req = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "john@example.com".to_string(),
        phone: "123".to_string(),
        role: UserRole::Organizer,
    };

    let result = service.create_user(req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "John");
}

#[tokio::test]
async fn test_create_user_empty_email() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));

    let req = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "".to_string(),
        phone: "123".to_string(),
        role: UserRole::Organizer,
    };

    let result = service.create_user(req).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Bad request: Email is required"
    );
}

// ── update_user ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_update_user_as_self() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![Model {
                id: "1".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
            vec![Model {
                id: "1".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
        ])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("1", "volunteer");

    let req = UpdateUserRequest {
        name: Some("Jane".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
    };

    let result = service.update_user("1", req, &claims).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "Jane");
}

#[tokio::test]
async fn test_update_user_as_admin() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![Model {
                id: "2".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
            vec![Model {
                id: "2".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
        ])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("admin-id", "admin");

    let req = UpdateUserRequest {
        name: Some("Jane".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
    };

    let result = service.update_user("2", req, &claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_user_as_organizer() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![Model {
                id: "2".to_string(),
                name: "John".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
            vec![Model {
                id: "2".to_string(),
                name: "Jane".to_string(),
                surname: "Doe".to_string(),
                email: "john@example.com".to_string(),
                phone: "123".to_string(),
                role: Role::Volunteer,
                status: Status::Active,
            }],
        ])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("org-id", "organizer");

    let req = UpdateUserRequest {
        name: Some("Jane".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
    };

    let result = service.update_user("2", req, &claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_update_user_forbidden() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("other-user", "volunteer");

    let req = UpdateUserRequest {
        name: Some("Hacked".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
    };

    let result = service.update_user("1", req, &claims).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Forbidden");
}

// ── delete_user ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_delete_user_as_self() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("1", "volunteer");

    let result = service.delete_user("1", &claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_user_as_admin() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 1,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("admin-id", "admin");

    let result = service.delete_user("1", &claims).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_delete_user_forbidden() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("other-user", "volunteer");

    let result = service.delete_user("1", &claims).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Forbidden");
}

#[tokio::test]
async fn test_delete_user_forbidden_as_organizer() {
    let db = setup_mock_db_with_user();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("org-id", "organizer");

    let result = service.delete_user("1", &claims).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Forbidden");
}

#[tokio::test]
async fn test_delete_user_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 0,
            rows_affected: 0,
        }])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));
    let claims = make_claims("1", "admin");

    let result = service.delete_user("1", &claims).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Not found: User not found");
}
