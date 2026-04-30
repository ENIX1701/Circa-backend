# Circa [Backend]

Rust backend for [Circa](https://github.com/ENIX1701/Circa). It exposes the event management API, handles magic-link authentication, issues JWTs, stores demo data in SQLite, and serves the frontend in production.

## Prerequisites

- `Rust` with `Cargo`
- SQLite runtime support through SeaORM/sqlx
- `pkg-config` and OpenSSL development libraries if your platform needs them for native TLS

## Run locally

Create `.env`:
```env
JWT_SECRET=change-me
DATABASE_URL=sqlite://data.db?mode=rwc
FRONTEND_URL=http://localhost:5173
AUTH_DELIVERY_MODE=outbox
APP_ENV=development
PORT=8080
```

Run:
```bash
cargo run
```

The API listens on:
```text
http://localhost:8080/api
```

## Environment variables

| Name                 | Type     | Default                              | Description                                      |
|----------------------|----------|--------------------------------------|--------------------------------------------------|
| `JWT_SECRET`         | `String` | required                             | Secret used to sign JWT auth tokens.            |
| `DATABASE_URL`       | `String` | `sqlite://data.db?mode=rwc` locally  | SeaORM database connection URL.                 |
| `FRONTEND_URL`       | `String` | `http://localhost:5173`              | Base URL used when generating magic links.      |
| `AUTH_DELIVERY_MODE` | `String` | `outbox`                             | `outbox` for test inbox, `smtp` placeholder.    |
| `APP_ENV`            | `String` | `development`                        | Set to `production` to serve the frontend SPA.  |
| `PORT`               | `u16`    | `8080`                               | HTTP port.                                      |

In production (heroku deploy), the default database URL becomes:
```text
sqlite:///tmp/circa.db?mode=rwc
```

## API overview

All routes are mounted under `/api`.

### Auth

| Method | Route                       | Description                       |
|--------|-----------------------------|-----------------------------------|
| `POST` | `/auth/request-link`        | Request a magic link for an email |
| `GET`  | `/auth/verify`              | Verify magic token and issue JWT  |
| `GET`  | `/auth/test-inbox/latest`   | Read latest outbox link by email  |
| `GET`  | `/me`                       | Return current JWT claims         |

### Users

Bearer auth required.

| Method   | Route         | Description       |
|----------|---------------|-------------------|
| `GET`    | `/users`      | List users        |
| `POST`   | `/users`      | Create user       |
| `GET`    | `/users/:id`  | Get user          |
| `PATCH`  | `/users/:id`  | Update user       |
| `DELETE` | `/users/:id`  | Delete user       |

### Events

Bearer auth required.

| Method   | Route                                      | Description                    |
|----------|--------------------------------------------|--------------------------------|
| `GET`    | `/events`                                  | List events for current user   |
| `POST`   | `/events`                                  | Create event                   |
| `GET`    | `/events/slug-availability`                | Check event slug availability  |
| `GET`    | `/events/:id`                              | Get event                      |
| `POST`   | `/events/:id/activate`                     | Activate event                 |
| `POST`   | `/events/:id/close`                        | Close event                    |
| `POST`   | `/events/:id/request-destruction`          | Request event destruction      |
| `POST`   | `/events/:id/cancel-destruction`           | Cancel destruction request     |
| `POST`   | `/events/:id/archive`                      | Archive event                  |
| `GET`    | `/events/:id/export`                       | Export event data              |
| `GET`    | `/events/:id/branding`                     | Get event branding             |
| `PUT`    | `/events/:id/branding`                     | Upsert event branding          |
| `GET`    | `/events/:id/social-posts`                 | List social posts              |
| `POST`   | `/events/:id/social-posts`                 | Create social post             |
| `PATCH`  | `/events/:id/social-posts/:post_id`        | Update social post             |
| `DELETE` | `/events/:id/social-posts/:post_id`        | Delete social post             |
| `GET`    | `/events/:id/planner-items`                | List planner checklist items   |
| `POST`   | `/events/:id/planner-items`                | Create planner checklist item  |
| `PATCH`  | `/events/:id/planner-items/:item_id`       | Update planner checklist item  |
| `DELETE` | `/events/:id/planner-items/:item_id`       | Delete planner checklist item  |
| `GET`    | `/events/:id/planner-timeline-items`       | List timeline items            |
| `POST`   | `/events/:id/planner-timeline-items`       | Create timeline item           |
| `PATCH`  | `/events/:id/planner-timeline-items/:id`   | Update timeline item           |
| `DELETE` | `/events/:id/planner-timeline-items/:id`   | Delete timeline item           |
| `GET`    | `/events/:id/collaborators`                | List event collaborators       |
| `POST`   | `/events/:id/collaborators`                | Add collaborator               |
| `PATCH`  | `/events/:id/collaborators/:user_id`       | Update collaborator role       |
| `DELETE` | `/events/:id/collaborators/:user_id`       | Remove collaborator            |

## Architecture

```text
src/
├── config.rs       # environment configuration
├── db.rs           # database connection and seed reset
├── error.rs        # shared API errors
├── main.rs         # Actix server entrypoint
├── models/         # shared backend models
└── modules/
    ├── auth/       # magic links, JWT, middleware, outbox
    ├── event/      # events, branding, planner, socials, collaborators
    └── user/       # users and role-aware access
```

Each domain module keeps its routes, service logic, entities, models, and repositories close together.

## Database

The backend uses SQLite through SeaORM.

Current tables:
- `users`
- `magic_tokens`
- `magic_link_outbox`
- `events`
- `event_memberships`
- `event_branding`
- `planner_items`
- `planner_timeline_items`
- `social_posts`

The application currently resets the database from `seed.sql` on startup and once per hour. This keeps the demo environment predictable, but it is not persistent production behavior. 

## Static serving

When `APP_ENV=production`, the backend serves the Vue SPA from:
```text
./public
```

The root Docker image copies `frontend/dist` there during the build.

## Checks

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
