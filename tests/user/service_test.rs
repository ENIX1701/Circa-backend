use circa_backend::user::{
    entity::{Model, Role, Status},
    models::{CreateUserRequest, UpdateUserRequest, UserRole},
    repository::UserRepository,
    service::UserService,
};
use sea_orm::MockDatabase;

fn setup_mock_db() -> sea_orm::DatabaseConnection {
    MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
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
        .into_connection()
}

#[tokio::test]
async fn test_create_user_success() {
    let db = setup_mock_db();
    let service = UserService::new(UserRepository::new(db));

    let req = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "john@example.com".to_string(),
        phone: "123".to_string(),
        role: UserRole::EventDirector,
    };

    let result = service.create_user(req).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().name, "John");
}

#[tokio::test]
async fn test_create_user_empty_email() {
    let db = setup_mock_db();
    let service = UserService::new(UserRepository::new(db));

    let req = CreateUserRequest {
        name: "John".to_string(),
        surname: "Doe".to_string(),
        email: "".to_string(),
        phone: "123".to_string(),
        role: UserRole::EventDirector,
    };

    let result = service.create_user(req).await;
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Bad request: Email is required"
    );
}

#[tokio::test]
async fn test_get_user_success() {
    let db = setup_mock_db();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user("1").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().id, "1");
}

#[tokio::test]
async fn test_get_user_not_found() {
    let db = MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
        .append_query_results([Vec::<Model>::new()])
        .into_connection();
    let service = UserService::new(UserRepository::new(db));

    let result = service.get_user("1234").await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Not found: User not found");
}

// #[tokio::test]
// async fn test_update_user_success() {
//     let db = MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
//         .append_query_results([
//             vec![Model {
//                 id: "1".to_string(),
//                 name: "John".to_string(),
//                 surname: "Doe".to_string(),
//                 email: "john@example.com".to_string(),
//                 phone: "123".to_string(),
//                 role: Role::EventDirector,
//                 status: Status::Active,
//             }],
//             vec![Model {
//                 id: "1".to_string(),
//                 name: "Jane".to_string(),
//                 surname: "Doe".to_string(),
//                 email: "john@example.com".to_string(),
//                 phone: "123".to_string(),
//                 role: Role::EventDirector,
//                 status: Status::Active,
//             }],
//         ])
//         .append_exec_results([sea_orm::MockExecResult {
//             last_insert_id: 0,
//             rows_affected: 1,
//         }])
//         .into_connection();
//     let service = UserService::new(UserRepository::new(db));

//     let req = UpdateUserRequest {
//         name: Some("Jane".to_string()),
//         surname: None,
//         email: None,
//         phone: None,
//         role: None,
//         status: None,
//     };

//     let result = service.update_user("1", req).await;
//     assert!(result.is_ok());
//     assert_eq!(result.unwrap().name, "Jane");
// }

// #[tokio::test]
// async fn test_delete_user_success() {
//     let db = MockDatabase::new(sea_orm::DatabaseBackend::Sqlite)
//         .append_exec_results([sea_orm::MockExecResult {
//             last_insert_id: 0,
//             rows_affected: 1,
//         }])
//         .into_connection();

//     let service = UserService::new(UserRepository::new(db));
//     let result = service.delete_user("1").await;

//     assert!(result.is_ok());
// }
