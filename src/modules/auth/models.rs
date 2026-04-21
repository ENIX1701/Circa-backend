use serde::{Deserialize, Serialize};

// === JWT ===
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

// === MAGIC LINK ===
// for now it'll follow a basic challenge/response model
// submit email to get magic link -> respond with magic link ->
// user responds with token -> if valid, gets JWT back
#[derive(Debug, Deserialize)]
pub struct MagicLinkRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct MagicLinkResponse {
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    pub token: String,
}

// === TEST INBOX ===
#[derive(Debug, Deserialize)]
pub struct TestInboxQuery {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct TestInboxLinkPreview {
    pub email: String,
    pub magic_link: String,
    pub requested_at: String,
    pub expires_at: String,
}
