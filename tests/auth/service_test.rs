use circa_backend::auth::models::Claims;
use circa_backend::auth::service::generate_jwt;
use jsonwebtoken::{DecodingKey, Validation, decode};

#[tokio::test]
async fn test_generate_jwt_success() {
    let secret = "test_secret";
    let result = generate_jwt("user@example.com", "admin", secret).await;

    assert!(result.is_ok());
    let token_response = result.unwrap();
    assert!(!token_response.token.is_empty());
}

#[tokio::test]
async fn test_generated_jwt_contains_correct_claims() {
    let secret = "test_secret";
    let email = "dave@example.com";
    let role = "organizer";

    let token_response = generate_jwt(email, role, secret).await.unwrap();

    let token_data = decode::<Claims>(
        &token_response.token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    assert_eq!(token_data.claims.sub, email);
    assert_eq!(token_data.claims.role, role);
    assert!(token_data.claims.exp > 0);
}

#[tokio::test]
async fn test_generated_jwt_invalid_with_wrong_secret() {
    let secret = "correct_secret";
    let token_response = generate_jwt("user@example.com", "admin", secret)
        .await
        .unwrap();

    let result = decode::<Claims>(
        &token_response.token,
        &DecodingKey::from_secret(b"wrong_secret"),
        &Validation::default(),
    );

    assert!(result.is_err());
}

#[tokio::test]
async fn test_generate_jwt_different_roles() {
    let secret = "test_secret";

    for role in &["admin", "organizer", "staff", "volunteer"] {
        let result = generate_jwt("user@example.com", role, secret).await;
        assert!(result.is_ok());

        let token_data = decode::<Claims>(
            &result.unwrap().token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        )
        .unwrap();

        assert_eq!(token_data.claims.role, *role);
    }
}
