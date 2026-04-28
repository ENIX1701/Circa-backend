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

    pub async fn get_users(&self, claims: &Claims) -> Result<Vec<User>, AppError> {
        Self::require_admin(claims)?;
        self.repository.find_all().await
    }

    pub async fn get_user(&self, id: &str, claims: &Claims) -> Result<User, AppError> {
        Self::require_self_or_admin(id, claims)?;

        let user = self.repository.find_by_id(id).await?;
        user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn get_user_for_auth(&self, id: &str) -> Result<User, AppError> {
        let user = self.repository.find_by_id(id).await?;
        user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn get_user_by_email(&self, email: &str) -> Result<User, AppError> {
        let user = self.repository.find_by_email(email).await?;
        user.ok_or_else(|| AppError::NotFound("User not found".to_string()))
    }

    pub async fn create_user(
        &self,
        req: CreateUserRequest,
        claims: &Claims,
    ) -> Result<User, AppError> {
        Self::require_admin(claims)?;

        if req.email.is_empty() {
            return Err(AppError::BadRequest("Email is required".to_string()));
        }

        self.repository.create(req).await
    }

    pub async fn update_user(
        &self,
        id: &str,
        mut req: UpdateUserRequest,
        claims: &Claims,
    ) -> Result<User, AppError> {
        Self::require_self_or_admin(id, claims)?;

        if claims.role != "admin" {
            req.role = None;
            req.status = None;
        }

        self.repository.update(id, req).await
    }

    pub async fn delete_user(&self, id: &str, claims: &Claims) -> Result<(), AppError> {
        Self::require_admin(claims)?;
        self.repository.delete(id).await
    }

    fn require_admin(claims: &Claims) -> Result<(), AppError> {
        if claims.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }

    fn require_self_or_admin(id: &str, claims: &Claims) -> Result<(), AppError> {
        if claims.sub == id || claims.role == "admin" {
            Ok(())
        } else {
            Err(AppError::Forbidden)
        }
    }
}
