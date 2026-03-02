use circa_backend::auth::models::Claims;
use circa_backend::auth::service::{create_magic_token, generate_jwt, verify_magic_token};
use circa_backend::modules::auth::entity::{self as magic_entity};
use jsonwebtoken::{DecodingKey, Validation, decode};
use sea_orm::{DatabaseBackend, MockDatabase};

// ── generate_jwt ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_generate_jwt_success() {
    let secret = "test_secret";
    let result = generate_jwt("user-123", "admin", secret).await;

    assert!(result.is_ok());
    let token_response = result.unwrap();
    assert!(!token_response.token.is_empty());
}

#[tokio::test]
async fn test_generated_jwt_contains_correct_claims() {
    let secret = "test_secret";
    let user_id = "user-456";
    let role = "organizer";

    let token_response = generate_jwt(user_id, role, secret).await.unwrap();

    let token_data = decode::<Claims>(
        &token_response.token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    assert_eq!(token_data.claims.sub, user_id);
    assert_eq!(token_data.claims.role, role);
    assert!(token_data.claims.exp > 0);
}

#[tokio::test]
async fn test_generated_jwt_invalid_with_wrong_secret() {
    let secret = "correct_secret";
    let token_response = generate_jwt("user-123", "admin", secret).await.unwrap();

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
        let result = generate_jwt("user-123", role, secret).await;
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

#[tokio::test]
async fn test_generate_jwt_expiration_is_in_the_future() {
    let secret = "test_secret";
    let token_response = generate_jwt("user-123", "admin", secret).await.unwrap();

    let token_data = decode::<Claims>(
        &token_response.token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .unwrap();

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    assert!(token_data.claims.exp > now);
    // Should be approximately 24 hours from now (86400 seconds)
    let diff = token_data.claims.exp - now;
    assert!(diff > 86000 && diff <= 86400);
}

// ── create_magic_token ───────────────────────────────────────────────

#[tokio::test]
async fn test_create_magic_token_success() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-123".to_string(),
            token: "some-token".to_string(),
            expires_at: chrono::Utc::now().to_rfc3339(),
            used: false,
        }]])
        .append_exec_results([sea_orm::MockExecResult {
            last_insert_id: 1,
            rows_affected: 1,
        }])
        .into_connection();

    let result = create_magic_token(&db, "user-123", "http://localhost:5137").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert!(!response.message.is_empty());
    assert!(response.message.contains("Magic link sent"));
}

#[tokio::test]
async fn test_create_magic_token_db_insert_failure() {
    // Provide no exec results so the insert fails
    let db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();

    let result = create_magic_token(&db, "user-123", "http://localhost:5137").await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Internal server error");
}

// ── verify_magic_token ───────────────────────────────────────────────

#[tokio::test]
async fn test_verify_magic_token_success() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            // First query: find the magic token record
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-123".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time.clone(),
                used: false,
            }],
            // Second query: update returns the updated model
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-123".to_string(),
                token: "valid-token-abc".to_string(),
                expires_at: future_time,
                used: true,
            }],
        ])
        .into_connection();

    let result = verify_magic_token(&db, "valid-token-abc").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "user-123");
}

#[tokio::test]
async fn test_verify_magic_token_not_found() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([Vec::<magic_entity::Model>::new()])
        .into_connection();

    let result = verify_magic_token(&db, "nonexistent-token").await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Bad request: Invalid or expired magic link"
    );
}

#[tokio::test]
async fn test_verify_magic_token_expired() {
    let past_time = (chrono::Utc::now() - chrono::Duration::minutes(30)).to_rfc3339();

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-123".to_string(),
            token: "expired-token".to_string(),
            expires_at: past_time,
            used: false,
        }]])
        .into_connection();

    let result = verify_magic_token(&db, "expired-token").await;

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "Bad request: Magic link has expired"
    );
}

#[tokio::test]
async fn test_verify_magic_token_invalid_date_format() {
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([vec![magic_entity::Model {
            id: "tok-1".to_string(),
            user_id: "user-123".to_string(),
            token: "bad-date-token".to_string(),
            expires_at: "not-a-valid-date".to_string(),
            used: false,
        }]])
        .into_connection();

    let result = verify_magic_token(&db, "bad-date-token").await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Internal server error");
}

#[tokio::test]
async fn test_verify_magic_token_db_query_failure() {
    // Empty mock with no query results causes a DB error
    let db = MockDatabase::new(DatabaseBackend::Sqlite).into_connection();

    let result = verify_magic_token(&db, "any-token").await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_verify_magic_token_update_failure() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(10)).to_rfc3339();

    // Provide first query result but no second query result so the update fails
    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-123".to_string(),
                token: "valid-token".to_string(),
                expires_at: future_time,
                used: false,
            }],
            Vec::<magic_entity::Model>::new(),
        ])
        .into_connection();

    let result = verify_magic_token(&db, "valid-token").await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().to_string(), "Internal server error");
}

#[tokio::test]
async fn test_verify_magic_token_returns_correct_user_id() {
    let future_time = (chrono::Utc::now() + chrono::Duration::minutes(5)).to_rfc3339();

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![magic_entity::Model {
                id: "tok-99".to_string(),
                user_id: "specific-user-id-abc".to_string(),
                token: "token-for-specific-user".to_string(),
                expires_at: future_time.clone(),
                used: false,
            }],
            vec![magic_entity::Model {
                id: "tok-99".to_string(),
                user_id: "specific-user-id-abc".to_string(),
                token: "token-for-specific-user".to_string(),
                expires_at: future_time,
                used: true,
            }],
        ])
        .into_connection();

    let result = verify_magic_token(&db, "token-for-specific-user").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "specific-user-id-abc");
}

#[tokio::test]
async fn test_verify_magic_token_just_before_expiry() {
    // Token expires 1 second from now - should still be valid
    let almost_expired = (chrono::Utc::now() + chrono::Duration::seconds(1)).to_rfc3339();

    let db = MockDatabase::new(DatabaseBackend::Sqlite)
        .append_query_results([
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-edge".to_string(),
                token: "almost-expired".to_string(),
                expires_at: almost_expired.clone(),
                used: false,
            }],
            vec![magic_entity::Model {
                id: "tok-1".to_string(),
                user_id: "user-edge".to_string(),
                token: "almost-expired".to_string(),
                expires_at: almost_expired,
                used: true,
            }],
        ])
        .into_connection();

    let result = verify_magic_token(&db, "almost-expired").await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "user-edge");
}
