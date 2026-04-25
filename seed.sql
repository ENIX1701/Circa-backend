-- dummy data for fe/be development
-- run with sqlite3 data.db < seed.sql

PRAGMA foreign_keys = ON;

BEGIN TRANSACTION;

-- === auth / staff ===
CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    surname TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT NOT NULL,
    availability_hours TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS magic_tokens (
    id TEXT PRIMARY KEY NOT NULL,
    user_id TEXT NOT NULL,
    token TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT 0
);

CREATE TABLE IF NOT EXISTS magic_link_outbox (
    id TEXT PRIMARY KEY NOT NULL,
    email TEXT NOT NULL,
    magic_token_id TEXT NOT NULL,
    magic_link TEXT NOT NULL,
    created_at TEXT NOT NULL,
    expires_at TEXT NOT NULL,
    used BOOLEAN NOT NULL DEFAULT 0
);

-- === events ===
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

CREATE TABLE IF NOT EXISTS event_memberships (
    id TEXT PRIMARY KEY NOT NULL,
    event_id TEXT NOT NULL,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL,
    created_at TEXT NOT NULL,
    UNIQUE(event_id, user_id)
);

CREATE TABLE IF NOT EXISTS event_branding (
    id TEXT PRIMARY KEY NOT NULL,
    event_id TEXT NOT NULL UNIQUE,
    event_name_override TEXT NOT NULL DEFAULT '',
    tagline TEXT NOT NULL DEFAULT '',
    primary_color TEXT NOT NULL DEFAULT '',
    secondary_color TEXT NOT NULL DEFAULT '',
    notes TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- === planner ===
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

CREATE INDEX IF NOT EXISTS idx_planner_items_event_id_position
    ON planner_items (event_id, position);

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

CREATE INDEX IF NOT EXISTS idx_planner_timeline_items_event_id_position
    ON planner_timeline_items (event_id, position);

-- === socials ===
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

CREATE INDEX IF NOT EXISTS idx_social_posts_event_id_position
    ON social_posts (event_id, position);

-- === clear previous dev seed rows ===
DELETE FROM social_posts
WHERE event_id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
);

DELETE FROM planner_timeline_items
WHERE event_id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
);

DELETE FROM planner_items
WHERE event_id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
);

DELETE FROM event_branding
WHERE event_id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
);

DELETE FROM event_memberships
WHERE event_id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
)
OR user_id IN (
    '019c8555-7a32-719a-bbfc-289d208c2996',
    '019c8555-7a32-7972-8961-f2c2b29ebd22',
    '019c8555-7a32-7aaa-8000-000000000003',
    '019c8555-7a32-7aaa-8000-000000000004'
);

DELETE FROM events
WHERE id IN (
    '019c8555-7a32-7aaa-8000-000000000100',
    '019c8555-7a32-7aaa-8000-000000000101'
)
OR slug IN ('circa-demo-2026', 'circa-afterparty-2026');

DELETE FROM magic_link_outbox
WHERE email IN (
    'alice@circa.local',
    'bob@circa.local',
    'cara@circa.local',
    'dani@circa.local'
)
OR magic_token_id IN (
    '019c8555-7a32-7aaa-8000-000000000201',
    '019c8555-7a32-7aaa-8000-000000000202'
);

DELETE FROM magic_tokens
WHERE user_id IN (
    '019c8555-7a32-719a-bbfc-289d208c2996',
    '019c8555-7a32-7972-8961-f2c2b29ebd22',
    '019c8555-7a32-7aaa-8000-000000000003',
    '019c8555-7a32-7aaa-8000-000000000004'
)
OR id IN (
    '019c8555-7a32-7aaa-8000-000000000201',
    '019c8555-7a32-7aaa-8000-000000000202'
);

DELETE FROM users
WHERE email IN (
    'alice@circa.local',
    'bob@circa.local',
    'cara@circa.local',
    'dani@circa.local'
)
OR id IN (
    '019c8555-7a32-719a-bbfc-289d208c2996',
    '019c8555-7a32-7972-8961-f2c2b29ebd22',
    '019c8555-7a32-7aaa-8000-000000000003',
    '019c8555-7a32-7aaa-8000-000000000004'
);

-- === seed users for each global role ===
INSERT INTO users (id, name, surname, email, phone, role, status, availability_hours) VALUES
    ('019c8555-7a32-719a-bbfc-289d208c2996', 'Alice', 'Lovelace', 'alice@circa.local', '+1-023-456-789', 'admin', 'active', 'Mon-Fri 09:00-18:00'),
    ('019c8555-7a32-7972-8961-f2c2b29ebd22', 'Bob', 'Birkenstock', 'bob@circa.local', '+1-321-654-987', 'organizer', 'active', 'Tue-Sat 10:00-17:00'),
    ('019c8555-7a32-7aaa-8000-000000000003', 'Cara', 'Curie', 'cara@circa.local', '+1-555-010-0003', 'staff', 'active', 'Fri 14:00-22:00, Sat 08:00-16:00'),
    ('019c8555-7a32-7aaa-8000-000000000004', 'Dani', 'Day', 'dani@circa.local', '+1-555-010-0004', 'volunteer', 'inactive', 'Sat 12:00-20:00');

-- === seed magic-link test inbox data ===
INSERT INTO magic_tokens (id, user_id, token, expires_at, used) VALUES
    ('019c8555-7a32-7aaa-8000-000000000201', '019c8555-7a32-719a-bbfc-289d208c2996', 'dev-alice-magic-token', '2026-12-31T23:59:59Z', 0),
    ('019c8555-7a32-7aaa-8000-000000000202', '019c8555-7a32-7972-8961-f2c2b29ebd22', 'dev-bob-used-magic-token', '2026-12-31T23:59:59Z', 1);

INSERT INTO magic_link_outbox (id, email, magic_token_id, magic_link, created_at, expires_at, used) VALUES
    ('019c8555-7a32-7aaa-8000-000000000211', 'alice@circa.local', '019c8555-7a32-7aaa-8000-000000000201', 'http://localhost:5173/auth/verify?token=dev-alice-magic-token', '2026-04-25T12:00:00Z', '2026-12-31T23:59:59Z', 0),
    ('019c8555-7a32-7aaa-8000-000000000212', 'bob@circa.local', '019c8555-7a32-7aaa-8000-000000000202', 'http://localhost:5173/auth/verify?token=dev-bob-used-magic-token', '2026-04-25T12:05:00Z', '2026-12-31T23:59:59Z', 1);

-- === seed events ===
INSERT INTO events (
    id, name, slug, description, venue, timezone, starts_at, ends_at,
    status, created_by_user_id, destruction_requested_at, created_at, updated_at
) VALUES
    (
        '019c8555-7a32-7aaa-8000-000000000100',
        'Circa Demo Summit 2026',
        'circa-demo-2026',
        'Development demo event with enough data to exercise dashboards, planner, socials, branding, memberships, and export.',
        'Warsaw Expo Hall A',
        'Europe/Warsaw',
        '2026-06-12T09:00:00+02:00',
        '2026-06-14T18:00:00+02:00',
        'active',
        '019c8555-7a32-719a-bbfc-289d208c2996',
        NULL,
        '2026-04-25T12:00:00Z',
        '2026-04-25T12:00:00Z'
    ),
    (
        '019c8555-7a32-7aaa-8000-000000000101',
        'Circa Afterparty 2026',
        'circa-afterparty-2026',
        'Secondary draft event for checking multi-event lists and inactive planning states.',
        'Riverside Warehouse',
        'Europe/Warsaw',
        '2026-06-14T20:00:00+02:00',
        '2026-06-15T02:00:00+02:00',
        'draft',
        '019c8555-7a32-7972-8961-f2c2b29ebd22',
        NULL,
        '2026-04-25T12:10:00Z',
        '2026-04-25T12:10:00Z'
    );

-- Event roles are scoped to a specific event and intentionally include owner.
INSERT INTO event_memberships (id, event_id, user_id, role, created_at) VALUES
    ('019c8555-7a32-7aaa-8000-000000000301', '019c8555-7a32-7aaa-8000-000000000100', '019c8555-7a32-719a-bbfc-289d208c2996', 'owner', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000302', '019c8555-7a32-7aaa-8000-000000000100', '019c8555-7a32-7972-8961-f2c2b29ebd22', 'organizer', '2026-04-25T12:01:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000303', '019c8555-7a32-7aaa-8000-000000000100', '019c8555-7a32-7aaa-8000-000000000003', 'staff', '2026-04-25T12:02:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000304', '019c8555-7a32-7aaa-8000-000000000100', '019c8555-7a32-7aaa-8000-000000000004', 'volunteer', '2026-04-25T12:03:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000305', '019c8555-7a32-7aaa-8000-000000000101', '019c8555-7a32-7972-8961-f2c2b29ebd22', 'owner', '2026-04-25T12:10:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000306', '019c8555-7a32-7aaa-8000-000000000101', '019c8555-7a32-719a-bbfc-289d208c2996', 'organizer', '2026-04-25T12:11:00Z');

INSERT INTO event_branding (
    id, event_id, event_name_override, tagline, primary_color, secondary_color,
    notes, created_at, updated_at
) VALUES
    (
        '019c8555-7a32-7aaa-8000-000000000401',
        '019c8555-7a32-7aaa-8000-000000000100',
        'Circa Demo Summit',
        'Ship the event, keep the chaos cute.',
        '#7C3AED',
        '#22D3EE',
        'Use the purple/cyan palette for landing pages, badges, and social previews.',
        '2026-04-25T12:00:00Z',
        '2026-04-25T12:00:00Z'
    ),
    (
        '019c8555-7a32-7aaa-8000-000000000402',
        '019c8555-7a32-7aaa-8000-000000000101',
        '',
        'Wind down together after the summit.',
        '#111827',
        '#F97316',
        'Draft palette; revisit once venue confirms lighting.',
        '2026-04-25T12:10:00Z',
        '2026-04-25T12:10:00Z'
    );

INSERT INTO planner_items (id, event_id, title, notes, position, done, created_at, updated_at) VALUES
    ('019c8555-7a32-7aaa-8000-000000000501', '019c8555-7a32-7aaa-8000-000000000100', 'Confirm venue floor plan', 'Upload latest hall map and mark registration, stage, green room, and sponsor booths.', 1, 1, '2026-04-25T12:00:00Z', '2026-04-25T12:30:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000502', '019c8555-7a32-7aaa-8000-000000000100', 'Publish volunteer rota', 'Check coverage for check-in, speakers, wayfinding, cloakroom, and teardown.', 2, 0, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000503', '019c8555-7a32-7aaa-8000-000000000100', 'Prepare export pack', 'Verify branding, planner, socials, and timeline are complete before sending to production.', 3, 0, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000504', '019c8555-7a32-7aaa-8000-000000000101', 'Confirm afterparty capacity', 'Ask venue for final fire-code capacity and wristband policy.', 1, 0, '2026-04-25T12:10:00Z', '2026-04-25T12:10:00Z');

INSERT INTO planner_timeline_items (
    id, event_id, title, item_type, starts_at, ends_at, status, owner,
    notes, color, position, created_at, updated_at, depends_on_item_id
) VALUES
    ('019c8555-7a32-7aaa-8000-000000000601', '019c8555-7a32-7aaa-8000-000000000100', 'Venue load-in', 'logistics', '2026-06-12T07:00:00+02:00', '2026-06-12T08:30:00+02:00', 'scheduled', 'Cara Curie', 'AV, signage, and registration desks arrive through Gate B.', '#22D3EE', 1, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z', ''),
    ('019c8555-7a32-7aaa-8000-000000000602', '019c8555-7a32-7aaa-8000-000000000100', 'Doors open and check-in', 'staffing', '2026-06-12T08:30:00+02:00', '2026-06-12T10:00:00+02:00', 'ready', 'Dani Day', 'Volunteer check-in team needs badge scanners and printed fallback list.', '#7C3AED', 2, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z', '019c8555-7a32-7aaa-8000-000000000601'),
    ('019c8555-7a32-7aaa-8000-000000000603', '019c8555-7a32-7aaa-8000-000000000100', 'Opening keynote', 'program', '2026-06-12T10:00:00+02:00', '2026-06-12T10:45:00+02:00', 'blocked', 'Alice Lovelace', 'Waiting on final speaker deck.', '#F97316', 3, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z', '019c8555-7a32-7aaa-8000-000000000602'),
    ('019c8555-7a32-7aaa-8000-000000000604', '019c8555-7a32-7aaa-8000-000000000100', 'Day-one wrap and handoff', 'operations', '2026-06-12T17:30:00+02:00', '2026-06-12T18:00:00+02:00', 'draft', 'Bob Birkenstock', 'Collect unresolved issues for day-two standup.', '#10B981', 4, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z', ''),
    ('019c8555-7a32-7aaa-8000-000000000605', '019c8555-7a32-7aaa-8000-000000000101', 'Afterparty doors open', 'social', '2026-06-14T20:00:00+02:00', '2026-06-14T21:00:00+02:00', 'draft', 'Bob Birkenstock', 'Depends on summit teardown plan.', '#F97316', 1, '2026-04-25T12:10:00Z', '2026-04-25T12:10:00Z', '');

INSERT INTO social_posts (id, event_id, platform, title, body, status, position, created_at, updated_at) VALUES
    ('019c8555-7a32-7aaa-8000-000000000701', '019c8555-7a32-7aaa-8000-000000000100', 'x', 'Speaker lineup teaser', 'First wave of speakers is nearly here. Follow Circa Demo Summit for schedule drops this week.', 'draft', 1, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000702', '019c8555-7a32-7aaa-8000-000000000100', 'linkedin', 'Volunteer call', 'We are opening a few more volunteer slots for registration, wayfinding, and speaker support.', 'scheduled', 2, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000703', '019c8555-7a32-7aaa-8000-000000000100', 'instagram', 'Behind the scenes', 'A peek at the venue build, badge wall, and stage tests before doors open.', 'published', 3, '2026-04-25T12:00:00Z', '2026-04-25T12:00:00Z'),
    ('019c8555-7a32-7aaa-8000-000000000704', '019c8555-7a32-7aaa-8000-000000000101', 'instagram', 'Afterparty save the date', 'Save the date for the Circa Afterparty. Details soon.', 'draft', 1, '2026-04-25T12:10:00Z', '2026-04-25T12:10:00Z');

COMMIT;

-- quick smoke-check for local CLI runs :3
SELECT 'users' AS table_name, COUNT(*) AS rows FROM users
UNION ALL SELECT 'magic_tokens', COUNT(*) FROM magic_tokens
UNION ALL SELECT 'magic_link_outbox', COUNT(*) FROM magic_link_outbox
UNION ALL SELECT 'events', COUNT(*) FROM events
UNION ALL SELECT 'event_memberships', COUNT(*) FROM event_memberships
UNION ALL SELECT 'event_branding', COUNT(*) FROM event_branding
UNION ALL SELECT 'planner_items', COUNT(*) FROM planner_items
UNION ALL SELECT 'planner_timeline_items', COUNT(*) FROM planner_timeline_items
UNION ALL SELECT 'social_posts', COUNT(*) FROM social_posts;
