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

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS events (
              id TEXT PRIMARY KEY NOT NULL,
              name TEXT NOT NULL,
              slug TEXT NOT NULL UNIQUE,
              description TEXT NOT NULL DEFAULT '',
              venue TEXT NOT NULL,
              timezone TEXT NOT NULL,
              starts_at TEXT NOT NULL,
              ends_at TEXT NOT NULL,
              status TEXT NOT NULL,
              created_by_user_id TEXT NOT NULL,
              destruction_requested_at TEXT NULL,
              created_at TEXT NOT NULL,
              updated_at TEXT NOT NULL
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS event_memberships (
              id TEXT PRIMARY KEY NOT NULL,
              event_id TEXT NOT NULL,
              user_id TEXT NOT NULL,
              role TEXT NOT NULL,
              created_at TEXT NOT NULL,
              UNIQUE(event_id, user_id)
            );
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS event_branding (
                id TEXT PRIMARY KEY NOT NULL,
                event_id TEXT NOT NULL UNIQUE,
                event_name_override TEXT NOT NULL DEFAULT '',
                tagline TEXT NOT NULL DEFAULT '',
                primary_color TEXT NOT NULL DEFAULT '',
                secondary_color TEXT NOT NULL DEFAULT '',
                theme_mode TEXT NOT NULL DEFAULT 'dark',
                background_color TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS planner_items (
                id TEXT PRIMARY KEY NOT NULL,
                event_id TEXT NOT NULL,
                title TEXT NOT NULL,
                notes TEXT NOT NULL DEFAULT '',
                position INTEGER NOT NULL,
                done BOOLEAN NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_planner_items_event_id_position
            ON planner_items (event_id, position);
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS planner_timeline_items (
                id TEXT PRIMARY KEY NOT NULL,
                event_id TEXT NOT NULL,
                title TEXT NOT NULL,
                item_type TEXT NOT NULL,
                starts_at TEXT NOT NULL,
                ends_at TEXT NOT NULL,
                status TEXT NOT NULL,
                owner TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                color TEXT NOT NULL DEFAULT '',
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                depends_on_item_id TEXT NOT NULL DEFAULT ''
            );
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_planner_timeline_items_event_id_position
            ON planner_timeline_items (event_id, position);
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE TABLE IF NOT EXISTS social_posts (
                id TEXT PRIMARY KEY NOT NULL,
                event_id TEXT NOT NULL,
                platform TEXT NOT NULL,
                title TEXT NOT NULL,
                body TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL,
                position INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            "#
        )
        .await?;

        db.execute_unprepared(
            r#"
            CREATE INDEX IF NOT EXISTS idx_social_posts_event_id_position
            ON social_posts (event_id, position);
            "#
        )
        .await?;
    }

    Ok(())
}
