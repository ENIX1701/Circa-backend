use actix_web::web;
use circa_backend::{
    auth::service::generate_jwt,
    config::{AppEnvironment, AuthDeliveryMode, Config},
    user::{repository::UserRepository, service::UserService},
};
use sea_orm::{ConnectOptions, ConnectionTrait, Database, DatabaseConnection};

pub const JWT_SECRET: &str = "test-secret";

pub fn test_config(auth_delivery_mode: AuthDeliveryMode) -> Config {
    Config {
        database_url: "sqlite::memory:".to_string(),
        jwt_secret: JWT_SECRET.to_string(),
        frontend_url: "http://localhost:5173".to_string(),
        app_env: AppEnvironment::Development,
        auth_delivery_mode,
    }
}

pub async fn jwt(sub: &str, role: &str) -> String {
    generate_jwt(sub, role, JWT_SECRET).await.unwrap().token
}

pub fn user_service(db: &DatabaseConnection) -> web::Data<UserService> {
    web::Data::new(UserService::new(UserRepository::new(db.clone())))
}

pub async fn setup_db() -> DatabaseConnection {
    let mut options = ConnectOptions::new("sqlite::memory:");
    options
        .max_connections(1)
        .min_connections(1)
        .sqlx_logging(false);

    let db = Database::connect(options).await.unwrap();

    for statement in [
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            surname TEXT NOT NULL,
            email TEXT NOT NULL UNIQUE,
            phone TEXT NOT NULL,
            role TEXT NOT NULL,
            status TEXT NOT NULL,
            availability_hours TEXT NOT NULL
        )
        "#,
        r#"
        CREATE TABLE magic_tokens (
            id TEXT PRIMARY KEY NOT NULL,
            user_id TEXT NOT NULL,
            token TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            used BOOLEAN NOT NULL DEFAULT 0
        )
        "#,
        r#"
        CREATE TABLE magic_link_outbox (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL,
            magic_token_id TEXT NOT NULL,
            magic_link TEXT NOT NULL,
            created_at TEXT NOT NULL,
            expires_at TEXT NOT NULL,
            used BOOLEAN NOT NULL DEFAULT 0
        )
        "#,
        r#"
        CREATE TABLE events (
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
        )
        "#,
        r#"
        CREATE TABLE event_memberships (
            id TEXT PRIMARY KEY NOT NULL,
            event_id TEXT NOT NULL,
            user_id TEXT NOT NULL,
            role TEXT NOT NULL,
            created_at TEXT NOT NULL,
            UNIQUE(event_id, user_id)
        )
        "#,
        r#"
        CREATE TABLE event_branding (
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
        )
        "#,
        r#"
        CREATE TABLE planner_items (
            id TEXT PRIMARY KEY NOT NULL,
            event_id TEXT NOT NULL,
            title TEXT NOT NULL,
            notes TEXT NOT NULL DEFAULT '',
            position INTEGER NOT NULL,
            done BOOLEAN NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
        r#"
        CREATE TABLE planner_timeline_items (
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
            depends_on_item_id TEXT NOT NULL DEFAULT '',
            assigned_user_id TEXT NOT NULL DEFAULT ''
        )
        "#,
        r#"
        CREATE TABLE social_posts (
            id TEXT PRIMARY KEY NOT NULL,
            event_id TEXT NOT NULL,
            platform TEXT NOT NULL,
            title TEXT NOT NULL,
            body TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL,
            position INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    ] {
        exec(&db, statement).await;
    }

    db
}

pub async fn exec(db: &DatabaseConnection, sql: &str) {
    db.execute_unprepared(sql).await.unwrap();
}

pub async fn seed_user(db: &DatabaseConnection, id: &str, email: &str, role: &str, status: &str) {
    exec(
        db,
        &format!(
            "INSERT INTO users (id, name, surname, email, phone, role, status, availability_hours)
             VALUES ('{id}', 'Test', 'User', '{email}', '+48123456789', '{role}', '{status}', '[]')"
        ),
    )
    .await;
}

pub async fn seed_event(db: &DatabaseConnection, id: &str, status: &str, creator: &str) {
    exec(
        db,
        &format!(
            "INSERT INTO events (
                id, name, slug, description, venue, timezone, starts_at, ends_at,
                status, created_by_user_id, destruction_requested_at, created_at, updated_at
            ) VALUES (
                '{id}', 'Circa Test Event', '{id}-slug', 'A test event', 'Expo Hall',
                'Europe/Warsaw', '2026-05-01T10:00:00Z', '2026-05-02T10:00:00Z',
                '{status}', '{creator}', NULL, '2026-04-20T10:00:00Z', '2026-04-20T10:00:00Z'
            )"
        ),
    )
    .await;
}

pub async fn seed_membership(db: &DatabaseConnection, event_id: &str, user_id: &str, role: &str) {
    exec(
        db,
        &format!(
            "INSERT INTO event_memberships (id, event_id, user_id, role, created_at)
             VALUES ('membership-{event_id}-{user_id}', '{event_id}', '{user_id}', '{role}', '2026-04-20T10:00:00Z')"
        ),
    )
    .await;
}

pub async fn seed_basic_event(db: &DatabaseConnection, event_id: &str, status: &str) {
    seed_user(db, "owner-1", "owner@circa.local", "admin", "active").await;
    seed_user(
        db,
        "organizer-1",
        "organizer@circa.local",
        "organizer",
        "active",
    )
    .await;
    seed_user(db, "staff-1", "staff@circa.local", "staff", "active").await;
    seed_user(
        db,
        "volunteer-1",
        "volunteer@circa.local",
        "volunteer",
        "active",
    )
    .await;

    seed_event(db, event_id, status, "owner-1").await;
    seed_membership(db, event_id, "owner-1", "owner").await;
    seed_membership(db, event_id, "organizer-1", "organizer").await;
    seed_membership(db, event_id, "staff-1", "staff").await;
    seed_membership(db, event_id, "volunteer-1", "volunteer").await;
}

pub async fn seed_planner_item(db: &DatabaseConnection, event_id: &str, id: &str, position: i32) {
    exec(
        db,
        &format!(
            "INSERT INTO planner_items (id, event_id, title, notes, position, done, created_at, updated_at)
             VALUES ('{id}', '{event_id}', 'Planner item', 'Notes', {position}, 0, '2026-04-20T10:00:00Z', '2026-04-20T10:00:00Z')"
        ),
    )
    .await;
}

pub async fn seed_timeline_item(db: &DatabaseConnection, event_id: &str, id: &str, position: i32) {
    exec(
        db,
        &format!(
            "INSERT INTO planner_timeline_items (
                id, event_id, title, item_type, starts_at, ends_at, status, owner, notes, color,
                position, created_at, updated_at, depends_on_item_id, assigned_user_id
            ) VALUES (
                '{id}', '{event_id}', 'Timeline item', 'task', '2026-05-01T10:00:00Z',
                '2026-05-01T12:00:00Z', 'planned', 'Owner', 'Notes', '#abcdef',
                {position}, '2026-04-20T10:00:00Z', '2026-04-20T10:00:00Z', '', 'staff-1'
            )"
        ),
    )
    .await;
}

pub async fn seed_social_post(db: &DatabaseConnection, event_id: &str, id: &str, position: i32) {
    exec(
        db,
        &format!(
            "INSERT INTO social_posts (id, event_id, platform, title, body, status, position, created_at, updated_at)
             VALUES ('{id}', '{event_id}', 'Mastodon', 'Post title', 'Post body', 'draft', {position}, '2026-04-20T10:00:00Z', '2026-04-20T10:00:00Z')"
        ),
    )
    .await;
}

pub async fn seed_branding(db: &DatabaseConnection, event_id: &str) {
    exec(
        db,
        &format!(
            "INSERT INTO event_branding (
                id, event_id, event_name_override, tagline, primary_color, secondary_color,
                theme_mode, background_color, notes, created_at, updated_at
            ) VALUES (
                'branding-{event_id}', '{event_id}', 'Override', 'Tagline', '#111111',
                '#222222', 'dark', '#333333', 'Notes', '2026-04-20T10:00:00Z',
                '2026-04-20T10:00:00Z'
            )"
        ),
    )
    .await;
}

pub async fn insert_magic_token(
    db: &DatabaseConnection,
    id: &str,
    user_id: &str,
    token: &str,
    expires_at: &str,
    used: bool,
) {
    exec(
        db,
        &format!(
            "INSERT INTO magic_tokens (id, user_id, token, expires_at, used)
             VALUES ('{id}', '{user_id}', '{token}', '{expires_at}', {})",
            if used { 1 } else { 0 }
        ),
    )
    .await;
}

pub struct OutboxSeed<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub magic_token_id: &'a str,
    pub link: &'a str,
    pub created_at: &'a str,
    pub expires_at: &'a str,
    pub used: bool,
}

pub async fn insert_outbox(db: &DatabaseConnection, seed: OutboxSeed<'_>) {
    exec(
        db,
        &format!(
            "INSERT INTO magic_link_outbox (id, email, magic_token_id, magic_link, created_at, expires_at, used)
             VALUES ('{}', '{}', '{}', '{}', '{}', '{}', {})",
            seed.id,
            seed.email,
            seed.magic_token_id,
            seed.link,
            seed.created_at,
            seed.expires_at,
            if seed.used { 1 } else { 0 }
        ),
    )
    .await;
}
