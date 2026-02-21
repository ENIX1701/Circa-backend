use circa_backend::modules::user::entity::{Model, Role, Status};
use circa_backend::modules::user::models::{User, UserRole, UserStatus};

#[test]
fn test_role_conversion() {
    assert_eq!(Role::from(UserRole::EventDirector), Role::EventDirector);
    assert_eq!(Role::from(UserRole::BoothOwner), Role::BoothOwner);
    assert_eq!(Role::from(UserRole::Clown), Role::Clown);

    assert_eq!(UserRole::from(Role::EventDirector), UserRole::EventDirector);
    assert_eq!(UserRole::from(Role::BoothOwner), UserRole::BoothOwner);
    assert_eq!(UserRole::from(Role::Clown), UserRole::Clown);
}

#[test]
fn test_status_conversion() {
    assert_eq!(Status::from(UserStatus::Active), Status::Active);
    assert_eq!(Status::from(UserStatus::Inactive), Status::Inactive);

    assert_eq!(UserStatus::from(Status::Active), UserStatus::Active);
    assert_eq!(UserStatus::from(Status::Inactive), UserStatus::Inactive);
}

#[test]
fn test_model_to_user_conversion() {
    let model = Model {
        id: "123".to_string(),
        name: "Dave".to_string(),
        surname: "Strider".to_string(),
        email: "dave@example.com".to_string(),
        phone: "123456789".to_string(),
        role: Role::Clown,
        status: Status::Active,
    };

    let user: User = model.into();

    assert_eq!(user.id, "123");
    assert_eq!(user.name, "Dave");
    assert_eq!(user.surname, "Strider");
    assert_eq!(user.email, "dave@example.com");
    assert_eq!(user.phone, "123456789");
    assert_eq!(user.role, UserRole::Clown);
    assert_eq!(user.status, UserStatus::Active);
}
