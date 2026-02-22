use super::models::User;
use super::repository::UserRepository;
use crate::auth::models::Claims;
use crate::error::AppError;
use crate::user::models::{CreateUserRequest, UpdateUserRequest};

pub struct UserService {
    repository: UserRepository,
}

impl UserService {
    pub fn new(repository: UserRepository) -> Self {
        Self { repository }
    }

    pub async fn get_users(&self) -> Result<Vec<User>, AppError> {
        self.repository.find_all().await
    }

    pub async fn get_user(&self, id: &str) -> Result<User, AppError> {
        let user = self.repository.find_by_id(id).await?;
        user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> Result<User, AppError> {
        if req.email.is_empty() {
            return Err(AppError::BadRequest("Email is required".to_string()));
        }

        self.repository.create(req).await
    }

    pub async fn update_user(
        &self,
        id: &str,
        req: UpdateUserRequest,
        claims: &Claims,
    ) -> Result<User, AppError> {
        if claims.sub != id && claims.role != "admin" && claims.role != "organizer" {
            return Err(AppError::Forbidden);
        }

        self.repository.update(id, req).await
    }

    pub async fn delete_user(&self, id: &str, claims: &Claims) -> Result<(), AppError> {
        if claims.sub != id && claims.role != "admin" {
            return Err(AppError::Forbidden);
        }

        self.repository.delete(id).await
    }
}
