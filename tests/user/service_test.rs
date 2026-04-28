use crate::common::{seed_user, setup_db};
use circa_backend::{
    auth::models::Claims,
    user::{
        entity::{Entity as UserEntity, Role, Status},
        models::{CreateUserRequest, UpdateUserRequest, UserRole, UserStatus},
        repository::UserRepository,
        service::UserService,
    },
};
use sea_orm::EntityTrait;

fn claims(sub: &str, role: &str) -> Claims {
    Claims {
        sub: sub.to_string(),
        role: role.to_string(),
        exp: 9_999_999_999,
    }
}

async fn service() -> (sea_orm::DatabaseConnection, UserService) {
    let db = setup_db().await;
    seed_user(&db, "admin-1", "admin@circa.local", "admin", "active").await;
    seed_user(&db, "user-1", "user@circa.local", "volunteer", "active").await;
    let service = UserService::new(UserRepository::new(db.clone()));
    (db, service)
}

#[actix_web::test]
async fn get_users_is_admin_only_and_returns_all_users() {
    let (_db, service) = service().await;

    let users = service
        .get_users(&claims("admin-1", "admin"))
        .await
        .unwrap();
    assert_eq!(users.len(), 2);

    let err = service
        .get_users(&claims("user-1", "volunteer"))
        .await
        .unwrap_err();
    assert_eq!(err.to_string(), "Forbidden");
}

#[actix_web::test]
async fn get_user_allows_self_or_admin_and_rejects_other_users() {
    let (_db, service) = service().await;

    assert_eq!(
        service
            .get_user("user-1", &claims("user-1", "volunteer"))
            .await
            .unwrap()
            .email,
        "user@circa.local"
    );
    assert_eq!(
        service
            .get_user("user-1", &claims("admin-1", "admin"))
            .await
            .unwrap()
            .id,
        "user-1"
    );
    assert_eq!(
        service
            .get_user("user-1", &claims("other", "volunteer"))
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );
}

#[actix_web::test]
async fn get_user_for_auth_and_email_lookup_bypass_request_claims() {
    let (_db, service) = service().await;

    assert_eq!(
        service.get_user_for_auth("user-1").await.unwrap().id,
        "user-1"
    );
    assert_eq!(
        service
            .get_user_by_email("user@circa.local")
            .await
            .unwrap()
            .id,
        "user-1"
    );

    assert_eq!(
        service
            .get_user_for_auth("missing")
            .await
            .unwrap_err()
            .to_string(),
        "Not found: User not found"
    );
    assert_eq!(
        service
            .get_user_by_email("missing@circa.local")
            .await
            .unwrap_err()
            .to_string(),
        "Not found: User not found"
    );
}

#[actix_web::test]
async fn create_user_is_admin_only_and_validates_email() {
    let (db, service) = service().await;

    let created = service
        .create_user(
            CreateUserRequest {
                name: "Alice".to_string(),
                surname: "Tester".to_string(),
                email: "alice@circa.local".to_string(),
                phone: "+48".to_string(),
                role: UserRole::Organizer,
                availability_hours: "[]".to_string(),
            },
            &claims("admin-1", "admin"),
        )
        .await
        .unwrap();

    assert_eq!(created.email, "alice@circa.local");
    assert_eq!(UserEntity::find().all(&db).await.unwrap().len(), 3);

    let forbidden = service
        .create_user(
            CreateUserRequest {
                name: "Bob".to_string(),
                surname: "Tester".to_string(),
                email: "bob@circa.local".to_string(),
                phone: "+48".to_string(),
                role: UserRole::Staff,
                availability_hours: "[]".to_string(),
            },
            &claims("user-1", "volunteer"),
        )
        .await
        .unwrap_err();
    assert_eq!(forbidden.to_string(), "Forbidden");

    let invalid = service
        .create_user(
            CreateUserRequest {
                name: "No".to_string(),
                surname: "Email".to_string(),
                email: "".to_string(),
                phone: "+48".to_string(),
                role: UserRole::Staff,
                availability_hours: "[]".to_string(),
            },
            &claims("admin-1", "admin"),
        )
        .await
        .unwrap_err();
    assert_eq!(invalid.to_string(), "Bad request: Email is required");
}

#[actix_web::test]
async fn update_user_allows_self_but_strips_privileged_fields_for_non_admins() {
    let (db, service) = service().await;

    let updated = service
        .update_user(
            "user-1",
            UpdateUserRequest {
                name: Some("Renamed".to_string()),
                surname: Some("Updated".to_string()),
                email: Some("renamed@circa.local".to_string()),
                phone: Some("+123".to_string()),
                role: Some(UserRole::Admin),
                status: Some(UserStatus::Inactive),
                availability_hours: Some("Fri".to_string()),
            },
            &claims("user-1", "volunteer"),
        )
        .await
        .unwrap();

    assert_eq!(updated.name, "Renamed");
    assert_eq!(updated.role, UserRole::Volunteer);
    assert_eq!(updated.status, UserStatus::Active);

    let stored = UserEntity::find_by_id("user-1")
        .one(&db)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(stored.role, Role::Volunteer);
    assert_eq!(stored.status, Status::Active);
}

#[actix_web::test]
async fn update_user_admin_can_change_role_and_status() {
    let (_db, service) = service().await;

    let updated = service
        .update_user(
            "user-1",
            UpdateUserRequest {
                name: None,
                surname: None,
                email: None,
                phone: None,
                role: Some(UserRole::Staff),
                status: Some(UserStatus::Inactive),
                availability_hours: None,
            },
            &claims("admin-1", "admin"),
        )
        .await
        .unwrap();

    assert_eq!(updated.role, UserRole::Staff);
    assert_eq!(updated.status, UserStatus::Inactive);
}

#[actix_web::test]
async fn update_user_rejects_other_users_and_missing_records() {
    let (_db, service) = service().await;

    let req = UpdateUserRequest {
        name: Some("Bad".to_string()),
        surname: None,
        email: None,
        phone: None,
        role: None,
        status: None,
        availability_hours: None,
    };

    assert_eq!(
        service
            .update_user("user-1", req, &claims("other", "volunteer"))
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );

    assert_eq!(
        service
            .update_user(
                "missing",
                UpdateUserRequest {
                    name: Some("Missing".to_string()),
                    surname: None,
                    email: None,
                    phone: None,
                    role: None,
                    status: None,
                    availability_hours: None,
                },
                &claims("admin-1", "admin"),
            )
            .await
            .unwrap_err()
            .to_string(),
        "Not found: User not found"
    );
}

#[actix_web::test]
async fn delete_user_is_admin_only() {
    let (db, service) = service().await;

    assert_eq!(
        service
            .delete_user("user-1", &claims("user-1", "volunteer"))
            .await
            .unwrap_err()
            .to_string(),
        "Forbidden"
    );

    service
        .delete_user("user-1", &claims("admin-1", "admin"))
        .await
        .unwrap();

    assert!(
        UserEntity::find_by_id("user-1")
            .one(&db)
            .await
            .unwrap()
            .is_none()
    );

    assert_eq!(
        service
            .delete_user("missing", &claims("admin-1", "admin"))
            .await
            .unwrap_err()
            .to_string(),
        "Not found: User not found"
    );
}
