use super::entity::{ActiveModel, Entity as UserEntity};
use super::models::{CreateUserRequest, User};
use crate::error::AppError;
use sea_orm::*;

pub struct UserRepository {
    db: DatabaseConnection,
}

impl UserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn find_all(&self) -> Result<Vec<User>, AppError> {
        let models = UserEntity::find()
            .all(&self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let model = UserEntity::find_by_id(id)
            .all(&self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(model.map(|m| m.into()))
    }

    pub async fn create(&self, dto: CreateUserRequest) -> Result<User, AppError> {
        let id = uuid::Uuid::new_v7().to_string();

        let new_user = ActiveModel {
            id: Set(id),
            name: Set(dto.name),
            surname: Set(dto.surname),
            email: Set(dto.email),
            phone: Set(dto.phone),
            role: Set(dto.role.into()),
            status: Set(super::entity::Status::Active),
        };

        let result = new_user
            .insert(&self.db)
            .await
            .map_err(|e| AppError::InternalServerError(e.to_string()))?;

        Ok(result.into())
    }
}
