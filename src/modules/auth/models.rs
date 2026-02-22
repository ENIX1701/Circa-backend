use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct TokenResponse {
    pub token: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

// TODO: check how things like these are even made
// ideally this would be a magic link sent to the user's inbox
// mail servers are a pain to set up correctly tho QwQ
pub struct ChallengeRequest {
    pub uhh: String,
}

pub struct ChallengeResponse {
    pub umm: String,
}
