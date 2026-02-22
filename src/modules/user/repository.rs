use super::entity::{ActiveModel, Entity as UserEntity};
use super::models::{CreateUserRequest, UpdateUserRequest, User};
use crate::error::AppError;
use sea_orm::*;
use uuid;

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
            .map_err(|_| AppError::InternalServerError)?;

        Ok(models.into_iter().map(|m| m.into()).collect())
    }

    pub async fn find_by_id(&self, id: &str) -> Result<Option<User>, AppError> {
        let model = UserEntity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(model.map(|m| m.into()))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let model = UserEntity::find()
            .filter(super::entity::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        Ok(model.map(|m| m.into()))
    }

    pub async fn create(&self, dto: CreateUserRequest) -> Result<User, AppError> {
        let id = uuid::Uuid::now_v7().to_string();

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
            .map_err(|_| AppError::InternalServerError)?;

        Ok(result.into())
    }

    pub async fn update(&self, id: &str, dto: UpdateUserRequest) -> Result<User, AppError> {
        let model = UserEntity::find_by_id(id.to_string())
            .one(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if let Some(model) = model {
            let mut active_model: ActiveModel = model.into();

            if let Some(name) = dto.name {
                active_model.name = Set(name);
            }
            if let Some(surname) = dto.surname {
                active_model.surname = Set(surname);
            }
            if let Some(email) = dto.email {
                active_model.email = Set(email);
            }
            if let Some(phone) = dto.phone {
                active_model.phone = Set(phone);
            }
            if let Some(role) = dto.role {
                active_model.role = Set(role.into());
            }
            if let Some(status) = dto.status {
                active_model.status = Set(status.into());
            }

            let result = active_model
                .update(&self.db)
                .await
                .map_err(|_| AppError::InternalServerError)?;

            Ok(result.into())
        } else {
            Err(AppError::NotFound("User not found".to_string()))
        }
    }

    pub async fn delete(&self, id: &str) -> Result<(), AppError> {
        let result = UserEntity::delete_by_id(id.to_string())
            .exec(&self.db)
            .await
            .map_err(|_| AppError::InternalServerError)?;

        if result.rows_affected == 0 {
            Err(AppError::NotFound("User not found".to_string()))
        } else {
            Ok(())
        }
    }
}
