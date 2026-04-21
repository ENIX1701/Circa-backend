use sea_orm::{ConnectionTrait, Database, DatabaseBackend, DatabaseConnection, DbErr};

pub async fn establish_connection(database_url: &str) -> Result<DatabaseConnection, DbErr> {
    let db = Database::connect(database_url).await?;
    initialize_schema(&db).await?;

    println!("Database connected successfully");
    Ok(db)
}

async fn initialize_schema(db: &DatabaseConnection) -> Result<(), DbErr> {
    if db.get_database_backend() == DatabaseBackend::Sqlite {
        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS magic_link_outbox (
              id TEXT PRIMARY KEY NOT NULL,
              email TEXT NOT NULL,
              magic_token_id TEXT NOT NULL,
              magic_link TEXT NOT NULL,
              created_at TEXT NOT NULL,
              expires_at TEXT NOT NULL,
              used BOOLEAN NOT NULL DEFAULT 0
            );
            "#,
        )
        .await?;
    }

    Ok(())
}
