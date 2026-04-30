-- Public demo data for Circa.
-- Run locally with: sqlite3 data.db < seed.sql

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
    theme_mode TEXT NOT NULL DEFAULT 'dark',
    background_color TEXT NOT NULL DEFAULT '',
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
    depends_on_item_id TEXT NOT NULL DEFAULT '',
    assigned_user_id TEXT NOT NULL DEFAULT ''
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

-- === reset mutable demo tables ===
DELETE FROM social_posts;
DELETE FROM planner_timeline_items;
DELETE FROM planner_items;
DELETE FROM event_branding;
DELETE FROM event_memberships;
DELETE FROM events;
DELETE FROM magic_link_outbox;
DELETE FROM magic_tokens;
DELETE FROM users;

-- === seed users for every global role and operating region ===
INSERT INTO users (id, name, surname, email, phone, role, status, availability_hours) VALUES
    ('00000000-0000-7000-8000-000000000001', 'Maya', 'Chen', 'maya.chen@circa.demo', '+1-415-555-0101', 'admin', 'active', 'Mon-Fri 08:00-18:00 PT, escalation on demo days'),
    ('00000000-0000-7000-8000-000000000002', 'Oliver', 'Grant', 'oliver.grant@circa.demo', '+44-20-5555-0102', 'organizer', 'active', 'Mon-Thu 09:00-17:00 GMT, Fri remote'),
    ('00000000-0000-7000-8000-000000000003', 'Priya', 'Nair', 'priya.nair@circa.demo', '+65-5555-0103', 'organizer', 'active', 'Tue-Sat 09:00-18:00 SGT'),
    ('00000000-0000-7000-8000-000000000004', 'Daniel', 'Okafor', 'daniel.okafor@circa.demo', '+254-20-555-0104', 'organizer', 'active', 'Mon-Fri 10:00-19:00 EAT'),
    ('00000000-0000-7000-8000-000000000005', 'Sofia', 'Marin', 'sofia.marin@circa.demo', '+34-91-555-0105', 'staff', 'active', 'Mon-Wed 08:00-16:00 CET, event weekends'),
    ('00000000-0000-7000-8000-000000000006', 'Liam', 'Connor', 'liam.connor@circa.demo', '+353-1-555-0106', 'staff', 'active', 'Tue-Sat 12:00-20:00 IST'),
    ('00000000-0000-7000-8000-000000000007', 'Amina', 'Hassan', 'amina.hassan@circa.demo', '+971-4-555-0107', 'staff', 'active', 'Sun-Thu 09:00-17:00 GST'),
    ('00000000-0000-7000-8000-000000000008', 'Noah', 'Smith', 'noah.smith@circa.demo', '+1-212-555-0108', 'staff', 'active', 'Mon-Fri 11:00-19:00 ET'),
    ('00000000-0000-7000-8000-000000000009', 'Elena', 'Rossi', 'elena.rossi@circa.demo', '+39-06-555-0109', 'staff', 'active', 'Mon-Fri 09:00-17:00 CET'),
    ('00000000-0000-7000-8000-000000000010', 'Kenji', 'Tanaka', 'kenji.tanaka@circa.demo', '+81-3-5555-0110', 'staff', 'active', 'Mon-Fri 10:00-18:00 JST'),
    ('00000000-0000-7000-8000-000000000011', 'Grace', 'Kim', 'grace.kim@circa.demo', '+82-2-555-0111', 'volunteer', 'active', 'Fri 16:00-22:00, Sat-Sun 09:00-18:00 KST'),
    ('00000000-0000-7000-8000-000000000012', 'Mateo', 'Silva', 'mateo.silva@circa.demo', '+55-11-5555-0112', 'volunteer', 'active', 'Thu-Sun 10:00-18:00 BRT'),
    ('00000000-0000-7000-8000-000000000013', 'Aisha', 'Khan', 'aisha.khan@circa.demo', '+92-21-555-0113', 'volunteer', 'active', 'Sat-Sun 08:00-20:00 PKT'),
    ('00000000-0000-7000-8000-000000000014', 'Lucas', 'Brown', 'lucas.brown@circa.demo', '+61-2-5555-0114', 'volunteer', 'active', 'Fri-Mon 09:00-17:00 AEST'),
    ('00000000-0000-7000-8000-000000000015', 'Emma', 'Hughes', 'emma.hughes@circa.demo', '+64-9-555-0115', 'volunteer', 'inactive', 'Inactive until July 2026'),
    ('00000000-0000-7000-8000-000000000016', 'Fatima', 'Al-Sayed', 'fatima.alsayed@circa.demo', '+974-5555-0116', 'staff', 'active', 'Sun-Thu 08:00-16:00 AST'),
    ('00000000-0000-7000-8000-000000000017', 'Julien', 'Martin', 'julien.martin@circa.demo', '+33-1-5555-0117', 'organizer', 'active', 'Mon-Fri 09:30-18:30 CET'),
    ('00000000-0000-7000-8000-000000000018', 'Wei', 'Zhang', 'wei.zhang@circa.demo', '+86-21-5555-0118', 'volunteer', 'active', 'Wed-Sun 13:00-21:00 CST'),
    ('00000000-0000-7000-8000-000000000019', 'Hannah', 'Mueller', 'hannah.mueller@circa.demo', '+49-30-5555-0119', 'staff', 'active', 'Mon-Fri 07:00-15:00 CET'),
    ('00000000-0000-7000-8000-000000000020', 'Sam', 'Rivera', 'sam.rivera@circa.demo', '+52-55-5555-0120', 'volunteer', 'inactive', 'Standby for remote moderation only');

-- === seed magic-link test inbox data ===
INSERT INTO magic_tokens (id, user_id, token, expires_at, used) VALUES
    ('00000000-0000-7000-8000-000000000201', '00000000-0000-7000-8000-000000000001', 'demo-maya-admin-token', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000202', '00000000-0000-7000-8000-000000000002', 'demo-oliver-organizer-token', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000203', '00000000-0000-7000-8000-000000000005', 'demo-sofia-staff-token', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000204', '00000000-0000-7000-8000-000000000011', 'demo-grace-volunteer-token', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000205', '00000000-0000-7000-8000-000000000003', 'demo-priya-used-token', '2027-12-31T23:59:59Z', 1),
    ('00000000-0000-7000-8000-000000000206', '00000000-0000-7000-8000-000000000004', 'demo-daniel-expired-token', '2026-01-31T23:59:59Z', 0);

INSERT INTO magic_link_outbox (id, email, magic_token_id, magic_link, created_at, expires_at, used) VALUES
    ('00000000-0000-7000-8000-000000000301', 'maya.chen@circa.demo', '00000000-0000-7000-8000-000000000201', '/login?token=demo-maya-admin-token', '2026-04-30T08:00:00Z', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000302', 'oliver.grant@circa.demo', '00000000-0000-7000-8000-000000000202', '/login?token=demo-oliver-organizer-token', '2026-04-30T08:05:00Z', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000303', 'sofia.marin@circa.demo', '00000000-0000-7000-8000-000000000203', '/login?token=demo-sofia-staff-token', '2026-04-30T08:10:00Z', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000304', 'grace.kim@circa.demo', '00000000-0000-7000-8000-000000000204', '/login?token=demo-grace-volunteer-token', '2026-04-30T08:15:00Z', '2027-12-31T23:59:59Z', 0),
    ('00000000-0000-7000-8000-000000000305', 'priya.nair@circa.demo', '00000000-0000-7000-8000-000000000205', '/login?token=demo-priya-used-token', '2026-04-29T18:00:00Z', '2027-12-31T23:59:59Z', 1),
    ('00000000-0000-7000-8000-000000000306', 'daniel.okafor@circa.demo', '00000000-0000-7000-8000-000000000206', '/login?token=demo-daniel-expired-token', '2026-01-30T18:00:00Z', '2026-01-31T23:59:59Z', 0);

-- === seed events across lifecycle states ===
INSERT INTO events (
    id, name, slug, description, venue, timezone, starts_at, ends_at,
    status, created_by_user_id, destruction_requested_at, created_at, updated_at
) VALUES
    (
        '00000000-0000-7000-8000-000000000101',
        'Global Product Launch Summit 2026',
        'global-product-launch-summit-2026',
        'A three-day hybrid launch program for product, marketing, partner, analyst, and press teams across North America, Europe, and Asia Pacific.',
        'Pier 27, San Francisco',
        'America/Los_Angeles',
        '2026-06-16T09:00:00-07:00',
        '2026-06-18T18:00:00-07:00',
        'active',
        '00000000-0000-7000-8000-000000000001',
        NULL,
        '2026-04-10T12:00:00Z',
        '2026-04-30T08:20:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000102',
        'Climate Finance Forum 2026',
        'climate-finance-forum-2026',
        'An executive forum connecting asset managers, public finance teams, climate scientists, and policy leads for practical transition-finance planning.',
        'Queen Elizabeth II Centre, London',
        'Europe/London',
        '2026-09-08T08:30:00+01:00',
        '2026-09-09T18:30:00+01:00',
        'draft',
        '00000000-0000-7000-8000-000000000002',
        NULL,
        '2026-04-12T09:30:00Z',
        '2026-04-30T08:25:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000103',
        'Asia Pacific Developer Week 2026',
        'asia-pacific-developer-week-2026',
        'A multi-track developer conference with workshops, hands-on labs, open-source maintainers, and technical community programming.',
        'Suntec Convention Centre, Singapore',
        'Asia/Singapore',
        '2026-08-03T09:00:00+08:00',
        '2026-08-07T17:30:00+08:00',
        'active',
        '00000000-0000-7000-8000-000000000003',
        NULL,
        '2026-04-14T07:45:00Z',
        '2026-04-30T08:30:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000104',
        'Humanitarian Logistics Expo 2026',
        'humanitarian-logistics-expo-2026',
        'A closed expo for emergency operations, nonprofit logistics, medical supply chains, field connectivity, and cross-border response teams.',
        'Kenyatta International Convention Centre, Nairobi',
        'Africa/Nairobi',
        '2026-03-10T08:00:00+03:00',
        '2026-03-12T18:00:00+03:00',
        'closed',
        '00000000-0000-7000-8000-000000000004',
        NULL,
        '2026-01-08T10:00:00Z',
        '2026-03-13T11:00:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000105',
        'Creator Economy Awards 2026',
        'creator-economy-awards-2026',
        'A finished awards show with sponsor activations, press interviews, live-stream production, finalist hosting, and post-event takedown data.',
        'Harbourfront Centre, Toronto',
        'America/Toronto',
        '2026-02-20T16:00:00-05:00',
        '2026-02-21T01:00:00-05:00',
        'pending_destruction',
        '00000000-0000-7000-8000-000000000017',
        '2026-04-28T15:00:00Z',
        '2025-12-18T14:00:00Z',
        '2026-04-28T15:00:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000106',
        'Civic Data Lab Europe 2026',
        'civic-data-lab-europe-2026',
        'An archived civic technology lab covering municipal open data, accessibility audits, participatory budgeting, and public-interest analytics.',
        'Barcelona International Convention Centre',
        'Europe/Madrid',
        '2026-01-21T09:00:00+01:00',
        '2026-01-23T17:30:00+01:00',
        'archived',
        '00000000-0000-7000-8000-000000000001',
        NULL,
        '2025-11-20T13:00:00Z',
        '2026-02-02T12:00:00Z'
    ),
    (
        '00000000-0000-7000-8000-000000000107',
        'Remote Operations Masterclass 2026',
        'remote-operations-masterclass-2026',
        'A digital-first training day for distributed operations, async incident response, virtual facilitation, and global support handoffs.',
        'Online with regional watch rooms',
        'UTC',
        '2026-11-04T10:00:00Z',
        '2026-11-04T20:00:00Z',
        'draft',
        '00000000-0000-7000-8000-000000000002',
        NULL,
        '2026-04-18T16:20:00Z',
        '2026-04-30T08:35:00Z'
    );

-- === event memberships and role coverage ===
INSERT INTO event_memberships (id, event_id, user_id, role, created_at) VALUES
    ('00000000-0000-7000-8000-000000000401', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000001', 'owner', '2026-04-10T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000402', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000002', 'organizer', '2026-04-10T12:05:00Z'),
    ('00000000-0000-7000-8000-000000000403', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000005', 'staff', '2026-04-10T12:10:00Z'),
    ('00000000-0000-7000-8000-000000000404', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000006', 'staff', '2026-04-10T12:15:00Z'),
    ('00000000-0000-7000-8000-000000000405', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000008', 'staff', '2026-04-10T12:20:00Z'),
    ('00000000-0000-7000-8000-000000000406', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000011', 'volunteer', '2026-04-10T12:25:00Z'),
    ('00000000-0000-7000-8000-000000000407', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000012', 'volunteer', '2026-04-10T12:30:00Z'),
    ('00000000-0000-7000-8000-000000000408', '00000000-0000-7000-8000-000000000101', '00000000-0000-7000-8000-000000000016', 'staff', '2026-04-10T12:35:00Z'),
    ('00000000-0000-7000-8000-000000000409', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000002', 'owner', '2026-04-12T09:30:00Z'),
    ('00000000-0000-7000-8000-000000000410', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000001', 'organizer', '2026-04-12T09:35:00Z'),
    ('00000000-0000-7000-8000-000000000411', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000003', 'organizer', '2026-04-12T09:40:00Z'),
    ('00000000-0000-7000-8000-000000000412', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000009', 'staff', '2026-04-12T09:45:00Z'),
    ('00000000-0000-7000-8000-000000000413', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000013', 'volunteer', '2026-04-12T09:50:00Z'),
    ('00000000-0000-7000-8000-000000000414', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000017', 'organizer', '2026-04-12T09:55:00Z'),
    ('00000000-0000-7000-8000-000000000415', '00000000-0000-7000-8000-000000000102', '00000000-0000-7000-8000-000000000020', 'volunteer', '2026-04-12T10:00:00Z'),
    ('00000000-0000-7000-8000-000000000416', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000003', 'owner', '2026-04-14T07:45:00Z'),
    ('00000000-0000-7000-8000-000000000417', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000004', 'organizer', '2026-04-14T07:50:00Z'),
    ('00000000-0000-7000-8000-000000000418', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000001', 'organizer', '2026-04-14T07:55:00Z'),
    ('00000000-0000-7000-8000-000000000419', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000007', 'staff', '2026-04-14T08:00:00Z'),
    ('00000000-0000-7000-8000-000000000420', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000010', 'staff', '2026-04-14T08:05:00Z'),
    ('00000000-0000-7000-8000-000000000421', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000012', 'volunteer', '2026-04-14T08:10:00Z'),
    ('00000000-0000-7000-8000-000000000422', '00000000-0000-7000-8000-000000000103', '00000000-0000-7000-8000-000000000018', 'volunteer', '2026-04-14T08:15:00Z'),
    ('00000000-0000-7000-8000-000000000423', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000004', 'owner', '2026-01-08T10:00:00Z'),
    ('00000000-0000-7000-8000-000000000424', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000001', 'organizer', '2026-01-08T10:05:00Z'),
    ('00000000-0000-7000-8000-000000000425', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000005', 'staff', '2026-01-08T10:10:00Z'),
    ('00000000-0000-7000-8000-000000000426', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000007', 'staff', '2026-01-08T10:15:00Z'),
    ('00000000-0000-7000-8000-000000000427', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000008', 'staff', '2026-01-08T10:20:00Z'),
    ('00000000-0000-7000-8000-000000000428', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000011', 'volunteer', '2026-01-08T10:25:00Z'),
    ('00000000-0000-7000-8000-000000000429', '00000000-0000-7000-8000-000000000104', '00000000-0000-7000-8000-000000000014', 'volunteer', '2026-01-08T10:30:00Z'),
    ('00000000-0000-7000-8000-000000000430', '00000000-0000-7000-8000-000000000105', '00000000-0000-7000-8000-000000000017', 'owner', '2025-12-18T14:00:00Z'),
    ('00000000-0000-7000-8000-000000000431', '00000000-0000-7000-8000-000000000105', '00000000-0000-7000-8000-000000000002', 'organizer', '2025-12-18T14:05:00Z'),
    ('00000000-0000-7000-8000-000000000432', '00000000-0000-7000-8000-000000000105', '00000000-0000-7000-8000-000000000009', 'staff', '2025-12-18T14:10:00Z'),
    ('00000000-0000-7000-8000-000000000433', '00000000-0000-7000-8000-000000000105', '00000000-0000-7000-8000-000000000015', 'volunteer', '2025-12-18T14:15:00Z'),
    ('00000000-0000-7000-8000-000000000434', '00000000-0000-7000-8000-000000000105', '00000000-0000-7000-8000-000000000020', 'volunteer', '2025-12-18T14:20:00Z'),
    ('00000000-0000-7000-8000-000000000435', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000001', 'owner', '2025-11-20T13:00:00Z'),
    ('00000000-0000-7000-8000-000000000436', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000003', 'organizer', '2025-11-20T13:05:00Z'),
    ('00000000-0000-7000-8000-000000000437', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000005', 'staff', '2025-11-20T13:10:00Z'),
    ('00000000-0000-7000-8000-000000000438', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000006', 'staff', '2025-11-20T13:15:00Z'),
    ('00000000-0000-7000-8000-000000000439', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000013', 'volunteer', '2025-11-20T13:20:00Z'),
    ('00000000-0000-7000-8000-000000000440', '00000000-0000-7000-8000-000000000106', '00000000-0000-7000-8000-000000000016', 'staff', '2025-11-20T13:25:00Z'),
    ('00000000-0000-7000-8000-000000000441', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000002', 'owner', '2026-04-18T16:20:00Z'),
    ('00000000-0000-7000-8000-000000000442', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000001', 'organizer', '2026-04-18T16:25:00Z'),
    ('00000000-0000-7000-8000-000000000443', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000010', 'staff', '2026-04-18T16:30:00Z'),
    ('00000000-0000-7000-8000-000000000444', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000011', 'volunteer', '2026-04-18T16:35:00Z'),
    ('00000000-0000-7000-8000-000000000445', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000014', 'volunteer', '2026-04-18T16:40:00Z'),
    ('00000000-0000-7000-8000-000000000446', '00000000-0000-7000-8000-000000000107', '00000000-0000-7000-8000-000000000018', 'volunteer', '2026-04-18T16:45:00Z');

INSERT INTO event_branding (
    id, event_id, event_name_override, tagline, primary_color, secondary_color,
    theme_mode, background_color, notes, created_at, updated_at
) VALUES
    ('00000000-0000-7000-8000-000000000501', '00000000-0000-7000-8000-000000000101', 'Global Product Launch Summit', 'Launch clearly. Coordinate globally.', '#0057D9', '#FFB000', 'dark', '#061A2F', 'Use high-contrast blue and amber across keynote lower thirds, partner signage, executive badges, and social announcement cards.', '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000502', '00000000-0000-7000-8000-000000000102', 'Climate Finance Forum', 'Capital planning for a lower-risk transition.', '#14532D', '#D97706', 'light', '#F8FAFC', 'Keep the visual language restrained, credible, and policy-friendly for institutional audiences.', '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000503', '00000000-0000-7000-8000-000000000103', 'APAC Developer Week', 'Build, benchmark, and ship together.', '#7C2D12', '#0EA5E9', 'dark', '#101828', 'Use warm technical accents with bright cyan for code labs, signage, and livestream overlays.', '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000504', '00000000-0000-7000-8000-000000000104', 'Humanitarian Logistics Expo', 'Operational readiness when the margin is thin.', '#0F766E', '#F59E0B', 'light', '#F9FAFB', 'Prioritize plain-language signage, accessibility, and multilingual information hierarchy.', '2026-01-08T10:00:00Z', '2026-03-13T11:00:00Z'),
    ('00000000-0000-7000-8000-000000000505', '00000000-0000-7000-8000-000000000105', 'Creator Economy Awards', 'Recognizing work that moved culture forward.', '#831843', '#F43F5E', 'dark', '#190A18', 'Archive final gala colors, sponsor lockups, finalist templates, and press-kit treatments.', '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000506', '00000000-0000-7000-8000-000000000106', 'Civic Data Lab Europe', 'Better public services through practical data work.', '#1D4ED8', '#14B8A6', 'light', '#EFF6FF', 'Archived theme retained for exported reports and public recap pages.', '2025-11-20T13:00:00Z', '2026-02-02T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000507', '00000000-0000-7000-8000-000000000107', 'Remote Operations Masterclass', 'Run calm, distributed operations across time zones.', '#4338CA', '#F97316', 'dark', '#0F172A', 'Use crisp operator-focused visuals for the digital classroom, recordings, and workbook templates.', '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z');

-- === planner checklists ===
INSERT INTO planner_items (id, event_id, title, notes, position, done, created_at, updated_at) VALUES
    ('00000000-0000-7000-8000-000000000601', '00000000-0000-7000-8000-000000000101', 'Finalize executive run of show', 'Confirm cue ownership for keynote, product reveal, analyst briefing, and partner handoff.', 1, 1, '2026-04-10T12:00:00Z', '2026-04-28T18:00:00Z'),
    ('00000000-0000-7000-8000-000000000602', '00000000-0000-7000-8000-000000000101', 'Lock sponsor booth floor plan', 'Assign booth numbers, power drops, demo tables, storage routes, and sponsor support contacts.', 2, 1, '2026-04-10T12:00:00Z', '2026-04-29T10:30:00Z'),
    ('00000000-0000-7000-8000-000000000603', '00000000-0000-7000-8000-000000000101', 'Prepare press embargo workflow', 'Coordinate badge access, media room signage, analyst seating, and embargo release timing.', 3, 0, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000604', '00000000-0000-7000-8000-000000000101', 'Rehearse livestream failover', 'Test primary stream, backup encoder, captions, recording archive, and remote speaker bridge.', 4, 0, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000605', '00000000-0000-7000-8000-000000000101', 'Confirm accessibility services', 'Verify live captions, quiet room, wheelchair routes, reserved seating, and dietary labels.', 5, 0, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000606', '00000000-0000-7000-8000-000000000101', 'Approve attendee email sequence', 'Review confirmation, travel reminder, agenda preview, and post-event survey copy.', 6, 1, '2026-04-10T12:00:00Z', '2026-04-26T15:45:00Z'),
    ('00000000-0000-7000-8000-000000000607', '00000000-0000-7000-8000-000000000101', 'Load product demo devices', 'Prepare laptops, spare cables, chargers, security tags, and device sign-out sheet.', 7, 0, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000608', '00000000-0000-7000-8000-000000000101', 'Print onsite escalation matrix', 'Include security, venue, medical, production, sponsor success, and executive contacts.', 8, 0, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000609', '00000000-0000-7000-8000-000000000102', 'Confirm investor roundtables', 'Balance region, sector, and asset-class representation across the invite-only sessions.', 1, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000610', '00000000-0000-7000-8000-000000000102', 'Validate policy speaker list', 'Check titles, affiliations, travel support, briefing format, and approval requirements.', 2, 1, '2026-04-12T09:30:00Z', '2026-04-24T13:15:00Z'),
    ('00000000-0000-7000-8000-000000000611', '00000000-0000-7000-8000-000000000102', 'Draft green-room briefings', 'Prepare one-page context notes for moderator angles and sensitive topics.', 3, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000612', '00000000-0000-7000-8000-000000000102', 'Review sponsor claims language', 'Ensure sustainability language is specific, substantiated, and approved by legal review.', 4, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000613', '00000000-0000-7000-8000-000000000102', 'Build delegate seating map', 'Separate competing firms, prioritize accessibility, and reserve media-observer row.', 5, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000614', '00000000-0000-7000-8000-000000000102', 'Prepare hybrid participation kit', 'Send platform links, timezone guidance, Q and A rules, and translation support details.', 6, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000615', '00000000-0000-7000-8000-000000000102', 'Define media access policy', 'Document which sessions are on record, background only, or closed-door.', 7, 1, '2026-04-12T09:30:00Z', '2026-04-28T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000616', '00000000-0000-7000-8000-000000000102', 'Schedule emissions-report review', 'Check venue, catering, travel, production, and waste assumptions for export notes.', 8, 0, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000617', '00000000-0000-7000-8000-000000000103', 'Publish workshop catalog', 'Confirm lab abstracts, prerequisites, capacity, waitlist rules, and repository links.', 1, 1, '2026-04-14T07:45:00Z', '2026-04-25T11:00:00Z'),
    ('00000000-0000-7000-8000-000000000618', '00000000-0000-7000-8000-000000000103', 'Procure lab cloud credits', 'Set per-attendee limits, abuse controls, billing alerts, and teardown automation.', 2, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000619', '00000000-0000-7000-8000-000000000103', 'Recruit hallway track hosts', 'Assign community moderators, accessibility support, and code-of-conduct contacts.', 3, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000620', '00000000-0000-7000-8000-000000000103', 'Finalize contributor badge rules', 'Define maintainer, speaker, sponsor engineer, student, and media badge variants.', 4, 1, '2026-04-14T07:45:00Z', '2026-04-28T12:40:00Z'),
    ('00000000-0000-7000-8000-000000000621', '00000000-0000-7000-8000-000000000103', 'Ship speaker technical checklist', 'Collect adapters, demo URLs, resolution needs, backup videos, and network requirements.', 5, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000622', '00000000-0000-7000-8000-000000000103', 'Arrange mentor office hours', 'Pair senior engineers with early-career attendees across tracks and time zones.', 6, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000623', '00000000-0000-7000-8000-000000000103', 'Test simultaneous interpretation', 'Confirm Mandarin, Japanese, Bahasa Indonesia, and English caption workflows.', 7, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000624', '00000000-0000-7000-8000-000000000103', 'Prepare open-source maintainer lounge', 'Stock signage, meeting pods, charging, refreshments, and conflict-resolution support.', 8, 0, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000625', '00000000-0000-7000-8000-000000000104', 'Archive exhibitor manifests', 'Store shipping records, customs notes, emergency contact sheets, and booth closeout forms.', 1, 1, '2026-01-08T10:00:00Z', '2026-03-12T20:00:00Z'),
    ('00000000-0000-7000-8000-000000000626', '00000000-0000-7000-8000-000000000104', 'Publish after-action report', 'Summarize attendance, supplier performance, field-readiness outcomes, and lessons learned.', 2, 1, '2026-01-08T10:00:00Z', '2026-03-20T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000627', '00000000-0000-7000-8000-000000000104', 'Close vendor invoices', 'Resolve outstanding freight, translation, booth build, and emergency medical invoices.', 3, 0, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z'),
    ('00000000-0000-7000-8000-000000000628', '00000000-0000-7000-8000-000000000104', 'Transfer attendee survey dataset', 'Export anonymized survey results for partner analysis and post-event report charts.', 4, 1, '2026-01-08T10:00:00Z', '2026-03-18T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000629', '00000000-0000-7000-8000-000000000104', 'Schedule partner debriefs', 'Book regional debrief calls with suppliers, NGOs, and government observers.', 5, 0, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z'),
    ('00000000-0000-7000-8000-000000000630', '00000000-0000-7000-8000-000000000104', 'Reconcile equipment returns', 'Track satellite kits, rugged tablets, signage cases, scanners, and translation headsets.', 6, 0, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z'),
    ('00000000-0000-7000-8000-000000000631', '00000000-0000-7000-8000-000000000105', 'Confirm data retention approval', 'Check which production files should be retained before final destruction.', 1, 0, '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000632', '00000000-0000-7000-8000-000000000105', 'Export sponsor deliverables', 'Package final impressions, stage photos, clip links, and invoice-ready proof documents.', 2, 1, '2025-12-18T14:00:00Z', '2026-02-25T10:00:00Z'),
    ('00000000-0000-7000-8000-000000000633', '00000000-0000-7000-8000-000000000105', 'Remove finalist PII from archive', 'Delete private travel, banking, and accommodation notes from retained award records.', 3, 0, '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000634', '00000000-0000-7000-8000-000000000105', 'Store winner release forms', 'Attach final signed releases to the export packet before destruction is completed.', 4, 1, '2025-12-18T14:00:00Z', '2026-02-22T11:00:00Z'),
    ('00000000-0000-7000-8000-000000000635', '00000000-0000-7000-8000-000000000106', 'Archive workshop templates', 'Move worksheets, datasets, facilitator notes, and accessibility checklists to long-term storage.', 1, 1, '2025-11-20T13:00:00Z', '2026-01-24T10:30:00Z'),
    ('00000000-0000-7000-8000-000000000636', '00000000-0000-7000-8000-000000000106', 'Publish civic impact recap', 'Summarize prototypes, municipal commitments, open-data releases, and public follow-ups.', 2, 1, '2025-11-20T13:00:00Z', '2026-02-01T16:00:00Z'),
    ('00000000-0000-7000-8000-000000000637', '00000000-0000-7000-8000-000000000106', 'Close grant reporting packet', 'Attach budget exports, attendee demographics, supplier diversity data, and evaluation notes.', 3, 1, '2025-11-20T13:00:00Z', '2026-02-02T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000638', '00000000-0000-7000-8000-000000000106', 'Tag reusable session recordings', 'Mark recordings suitable for public release, internal training, or partner-only sharing.', 4, 1, '2025-11-20T13:00:00Z', '2026-01-29T14:00:00Z'),
    ('00000000-0000-7000-8000-000000000639', '00000000-0000-7000-8000-000000000107', 'Design virtual classroom flow', 'Map facilitator cues, breakout timing, support-room ownership, and global handoff windows.', 1, 0, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000640', '00000000-0000-7000-8000-000000000107', 'Prepare operations workbook', 'Write templates for incident notes, shift handoffs, escalation trees, and async status updates.', 2, 0, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000641', '00000000-0000-7000-8000-000000000107', 'Record platform walkthrough', 'Capture login, agenda, captions, question queue, breakout, and help-desk flows.', 3, 0, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000642', '00000000-0000-7000-8000-000000000107', 'Recruit regional watch-room hosts', 'Assign hosts for Americas, EMEA, and APAC practice groups.', 4, 0, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000643', '00000000-0000-7000-8000-000000000107', 'Define completion certificate rules', 'Confirm attendance threshold, exercise submission, and name-format policy.', 5, 1, '2026-04-18T16:20:00Z', '2026-04-29T16:00:00Z'),
    ('00000000-0000-7000-8000-000000000644', '00000000-0000-7000-8000-000000000107', 'Test backup facilitation channel', 'Validate chat bridge, phone bridge, status page, and moderator escalation flow.', 6, 0, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z');

-- === planner timeline / gantt data ===
INSERT INTO planner_timeline_items (
    id, event_id, title, item_type, starts_at, ends_at, status, owner,
    notes, color, position, created_at, updated_at, depends_on_item_id, assigned_user_id
) VALUES
    ('00000000-0000-7000-8000-000000000701', '00000000-0000-7000-8000-000000000101', 'Venue contract signed', 'milestone', '2026-06-10T09:00:00-07:00', '2026-06-10T09:00:00-07:00', 'done', 'Maya Chen', 'Final amendment includes livestream rigging and extended load-out.', '#FFB000', 1, '2026-04-10T12:00:00Z', '2026-04-28T18:00:00Z', '', '00000000-0000-7000-8000-000000000001'),
    ('00000000-0000-7000-8000-000000000702', '00000000-0000-7000-8000-000000000101', 'Sponsor asset collection', 'asset', '2026-06-11T09:00:00-07:00', '2026-06-13T17:00:00-07:00', 'in_progress', 'Oliver Grant', 'Collect logo packs, booth copy, session slides, and invoice references.', '#0057D9', 2, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '00000000-0000-7000-8000-000000000701', '00000000-0000-7000-8000-000000000002'),
    ('00000000-0000-7000-8000-000000000703', '00000000-0000-7000-8000-000000000101', 'Registration desk build', 'task', '2026-06-15T07:00:00-07:00', '2026-06-15T13:00:00-07:00', 'planned', 'Sofia Marin', 'Place scanners, badge stock, visa-letter pickup, and accessibility desk signage.', '#10B981', 3, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000005'),
    ('00000000-0000-7000-8000-000000000704', '00000000-0000-7000-8000-000000000101', 'Demo device imaging', 'task', '2026-06-15T09:00:00-07:00', '2026-06-16T12:00:00-07:00', 'blocked', 'Liam Connor', 'Waiting on final build from product engineering.', '#DC2626', 4, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000006'),
    ('00000000-0000-7000-8000-000000000705', '00000000-0000-7000-8000-000000000101', 'Executive rehearsal', 'task', '2026-06-15T14:00:00-07:00', '2026-06-15T18:00:00-07:00', 'planned', 'Maya Chen', 'Dry run keynote, launch reveal, analyst Q and A, and speaker transitions.', '#7C3AED', 5, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '00000000-0000-7000-8000-000000000704', '00000000-0000-7000-8000-000000000001'),
    ('00000000-0000-7000-8000-000000000706', '00000000-0000-7000-8000-000000000101', 'Doors open', 'milestone', '2026-06-16T09:00:00-07:00', '2026-06-16T09:00:00-07:00', 'planned', 'Sofia Marin', 'Front-of-house, sponsor desk, and media room open together.', '#FFB000', 6, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '00000000-0000-7000-8000-000000000703', '00000000-0000-7000-8000-000000000005'),
    ('00000000-0000-7000-8000-000000000707', '00000000-0000-7000-8000-000000000101', 'Partner showcase operations', 'task', '2026-06-16T10:00:00-07:00', '2026-06-17T17:00:00-07:00', 'planned', 'Noah Smith', 'Rotate sponsor success, booth troubleshooting, lead scanning, and demo support.', '#0EA5E9', 7, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000008'),
    ('00000000-0000-7000-8000-000000000708', '00000000-0000-7000-8000-000000000101', 'Accessibility services live', 'task', '2026-06-16T08:00:00-07:00', '2026-06-18T18:00:00-07:00', 'planned', 'Fatima Al-Sayed', 'Monitor captions, reserved seating, quiet room, and accommodation desk.', '#14B8A6', 8, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000016'),
    ('00000000-0000-7000-8000-000000000709', '00000000-0000-7000-8000-000000000101', 'Launch keynote', 'milestone', '2026-06-16T10:00:00-07:00', '2026-06-16T10:00:00-07:00', 'planned', 'Maya Chen', 'Main stage reveal and partner announcement.', '#FFB000', 9, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '00000000-0000-7000-8000-000000000705', '00000000-0000-7000-8000-000000000001'),
    ('00000000-0000-7000-8000-000000000710', '00000000-0000-7000-8000-000000000101', 'Post-launch media room', 'task', '2026-06-16T11:00:00-07:00', '2026-06-16T16:00:00-07:00', 'planned', 'Grace Kim', 'Escort media, maintain interview queue, and log embargo questions.', '#6366F1', 10, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '00000000-0000-7000-8000-000000000709', '00000000-0000-7000-8000-000000000011'),
    ('00000000-0000-7000-8000-000000000711', '00000000-0000-7000-8000-000000000101', 'Customer advisory dinner', 'task', '2026-06-17T18:00:00-07:00', '2026-06-17T22:00:00-07:00', 'planned', 'Mateo Silva', 'Coordinate transport, host seating, dietary flags, and executive arrivals.', '#F97316', 11, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000012'),
    ('00000000-0000-7000-8000-000000000712', '00000000-0000-7000-8000-000000000101', 'Strike and equipment audit', 'task', '2026-06-18T18:00:00-07:00', '2026-06-18T23:00:00-07:00', 'planned', 'Liam Connor', 'Check demo devices, signage cases, sponsor assets, and freight pickup.', '#64748B', 12, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z', '', '00000000-0000-7000-8000-000000000006'),
    ('00000000-0000-7000-8000-000000000713', '00000000-0000-7000-8000-000000000102', 'Venue shortlist approved', 'milestone', '2026-07-15T09:00:00+01:00', '2026-07-15T09:00:00+01:00', 'done', 'Oliver Grant', 'Preferred venue can support plenary, roundtables, and hybrid production.', '#D97706', 1, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000002'),
    ('00000000-0000-7000-8000-000000000714', '00000000-0000-7000-8000-000000000102', 'Speaker invitation wave one', 'task', '2026-07-20T09:00:00+01:00', '2026-07-31T17:00:00+01:00', 'in_progress', 'Priya Nair', 'Invite central banks, transition-risk leads, and infrastructure investors.', '#14532D', 2, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '00000000-0000-7000-8000-000000000713', '00000000-0000-7000-8000-000000000003'),
    ('00000000-0000-7000-8000-000000000715', '00000000-0000-7000-8000-000000000102', 'Sponsor claims review', 'asset', '2026-08-01T09:00:00+01:00', '2026-08-15T17:00:00+01:00', 'blocked', 'Julien Martin', 'Legal review is waiting on substantiation notes from two sponsors.', '#DC2626', 3, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000017'),
    ('00000000-0000-7000-8000-000000000716', '00000000-0000-7000-8000-000000000102', 'Delegate briefing pack', 'asset', '2026-08-12T09:00:00+01:00', '2026-08-28T17:00:00+01:00', 'planned', 'Elena Rossi', 'Prepare agenda, attendee profile, arrival guidance, and policy glossary.', '#0F766E', 4, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000009'),
    ('00000000-0000-7000-8000-000000000717', '00000000-0000-7000-8000-000000000102', 'Roundtable host training', 'task', '2026-09-01T10:00:00+01:00', '2026-09-02T16:00:00+01:00', 'planned', 'Maya Chen', 'Train hosts on Chatham House rules, timekeeping, and escalation paths.', '#1D4ED8', 5, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000001'),
    ('00000000-0000-7000-8000-000000000718', '00000000-0000-7000-8000-000000000102', 'Forum opens', 'milestone', '2026-09-08T08:30:00+01:00', '2026-09-08T08:30:00+01:00', 'planned', 'Oliver Grant', 'Delegate check-in and press desk go live.', '#D97706', 6, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '00000000-0000-7000-8000-000000000717', '00000000-0000-7000-8000-000000000002'),
    ('00000000-0000-7000-8000-000000000719', '00000000-0000-7000-8000-000000000102', 'Closed-door investor sessions', 'task', '2026-09-08T13:00:00+01:00', '2026-09-09T16:00:00+01:00', 'planned', 'Aisha Khan', 'Manage room turns, attendee lists, privacy signs, and note courier flow.', '#14B8A6', 7, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000013'),
    ('00000000-0000-7000-8000-000000000720', '00000000-0000-7000-8000-000000000102', 'Emissions data export', 'task', '2026-09-10T10:00:00+01:00', '2026-09-12T17:00:00+01:00', 'planned', 'Elena Rossi', 'Compile venue, travel, catering, and production assumptions for the event export.', '#84CC16', 8, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z', '', '00000000-0000-7000-8000-000000000009'),
    ('00000000-0000-7000-8000-000000000721', '00000000-0000-7000-8000-000000000103', 'Call for proposals closes', 'milestone', '2026-05-20T23:00:00+08:00', '2026-05-20T23:00:00+08:00', 'in_progress', 'Priya Nair', 'Final reminder campaign is scheduled across developer communities.', '#0EA5E9', 1, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000003'),
    ('00000000-0000-7000-8000-000000000722', '00000000-0000-7000-8000-000000000103', 'Workshop environment build', 'task', '2026-07-20T09:00:00+08:00', '2026-07-31T18:00:00+08:00', 'blocked', 'Kenji Tanaka', 'Waiting for final cloud quota and classroom account approval.', '#DC2626', 2, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000010'),
    ('00000000-0000-7000-8000-000000000723', '00000000-0000-7000-8000-000000000103', 'Open-source maintainer travel', 'asset', '2026-07-22T09:00:00+08:00', '2026-08-01T12:00:00+08:00', 'in_progress', 'Daniel Okafor', 'Track visa letters, hotel blocks, arrival windows, and speaker reimbursements.', '#7C2D12', 3, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000004'),
    ('00000000-0000-7000-8000-000000000724', '00000000-0000-7000-8000-000000000103', 'Community code-of-conduct desk', 'task', '2026-08-03T08:00:00+08:00', '2026-08-07T17:30:00+08:00', 'planned', 'Amina Hassan', 'Staff confidential reporting desk and hallway moderation rotations.', '#14B8A6', 4, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000007'),
    ('00000000-0000-7000-8000-000000000725', '00000000-0000-7000-8000-000000000103', 'Developer Week opens', 'milestone', '2026-08-03T09:00:00+08:00', '2026-08-03T09:00:00+08:00', 'planned', 'Priya Nair', 'Keynote and community welcome begin the week.', '#0EA5E9', 5, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '00000000-0000-7000-8000-000000000724', '00000000-0000-7000-8000-000000000003'),
    ('00000000-0000-7000-8000-000000000726', '00000000-0000-7000-8000-000000000103', 'Hands-on lab operations', 'task', '2026-08-03T10:00:00+08:00', '2026-08-06T17:00:00+08:00', 'planned', 'Kenji Tanaka', 'Monitor capacity, cloud credentials, lab assistants, and instructor escalations.', '#F97316', 6, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '00000000-0000-7000-8000-000000000722', '00000000-0000-7000-8000-000000000010'),
    ('00000000-0000-7000-8000-000000000727', '00000000-0000-7000-8000-000000000103', 'Mentor office hours', 'task', '2026-08-04T13:00:00+08:00', '2026-08-07T16:00:00+08:00', 'planned', 'Mateo Silva', 'Route early-career attendees to mentors by topic and language preference.', '#6366F1', 7, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000012'),
    ('00000000-0000-7000-8000-000000000728', '00000000-0000-7000-8000-000000000103', 'Closing maintainer panel', 'milestone', '2026-08-07T16:00:00+08:00', '2026-08-07T16:00:00+08:00', 'planned', 'Amina Hassan', 'Panel recording and transcript feed the recap campaign.', '#0EA5E9', 8, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z', '', '00000000-0000-7000-8000-000000000007'),
    ('00000000-0000-7000-8000-000000000729', '00000000-0000-7000-8000-000000000104', 'Freight arrival window', 'task', '2026-03-08T07:00:00+03:00', '2026-03-09T19:00:00+03:00', 'done', 'Amina Hassan', 'Satellite kits, medical demo cases, and booth supplies cleared receiving.', '#0F766E', 1, '2026-01-08T10:00:00Z', '2026-03-09T20:00:00Z', '', '00000000-0000-7000-8000-000000000007'),
    ('00000000-0000-7000-8000-000000000730', '00000000-0000-7000-8000-000000000104', 'Emergency response tabletop', 'milestone', '2026-03-10T11:00:00+03:00', '2026-03-10T11:00:00+03:00', 'done', 'Daniel Okafor', 'Cross-functional scenario drill completed with observer notes.', '#F59E0B', 2, '2026-01-08T10:00:00Z', '2026-03-10T12:30:00Z', '', '00000000-0000-7000-8000-000000000004'),
    ('00000000-0000-7000-8000-000000000731', '00000000-0000-7000-8000-000000000104', 'Medical supply chain demos', 'task', '2026-03-10T13:00:00+03:00', '2026-03-11T17:00:00+03:00', 'done', 'Noah Smith', 'Demo rotations covered cold-chain, customs, and last-mile inventory flows.', '#14B8A6', 3, '2026-01-08T10:00:00Z', '2026-03-11T18:00:00Z', '00000000-0000-7000-8000-000000000730', '00000000-0000-7000-8000-000000000008'),
    ('00000000-0000-7000-8000-000000000732', '00000000-0000-7000-8000-000000000104', 'Translation headset returns', 'asset', '2026-03-12T17:00:00+03:00', '2026-03-13T12:00:00+03:00', 'blocked', 'Sofia Marin', 'Eight headsets are still marked as unreturned by supplier manifest.', '#DC2626', 4, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z', '', '00000000-0000-7000-8000-000000000005'),
    ('00000000-0000-7000-8000-000000000733', '00000000-0000-7000-8000-000000000104', 'Partner after-action interviews', 'task', '2026-03-18T09:00:00+03:00', '2026-03-22T17:00:00+03:00', 'in_progress', 'Grace Kim', 'Collect partner quotes, operational gaps, and follow-up commitments.', '#1D4ED8', 5, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z', '', '00000000-0000-7000-8000-000000000011'),
    ('00000000-0000-7000-8000-000000000734', '00000000-0000-7000-8000-000000000104', 'Final invoice reconciliation', 'task', '2026-03-25T09:00:00+03:00', '2026-04-12T17:00:00+03:00', 'blocked', 'Daniel Okafor', 'Freight surcharge dispute remains open with vendor finance.', '#DC2626', 6, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z', '', '00000000-0000-7000-8000-000000000004'),
    ('00000000-0000-7000-8000-000000000735', '00000000-0000-7000-8000-000000000105', 'Finalist check-in desk', 'task', '2026-02-20T15:00:00-05:00', '2026-02-20T19:00:00-05:00', 'done', 'Elena Rossi', 'Finalists, guests, and press were checked in before red carpet close.', '#F43F5E', 1, '2025-12-18T14:00:00Z', '2026-02-20T19:30:00Z', '', '00000000-0000-7000-8000-000000000009'),
    ('00000000-0000-7000-8000-000000000736', '00000000-0000-7000-8000-000000000105', 'Winner envelope custody', 'milestone', '2026-02-20T20:00:00-05:00', '2026-02-20T20:00:00-05:00', 'done', 'Julien Martin', 'Custody log closed after final category announcement.', '#F43F5E', 2, '2025-12-18T14:00:00Z', '2026-02-21T02:00:00Z', '', '00000000-0000-7000-8000-000000000017'),
    ('00000000-0000-7000-8000-000000000737', '00000000-0000-7000-8000-000000000105', 'Sponsor deliverables archive', 'asset', '2026-02-22T09:00:00-05:00', '2026-02-28T17:00:00-05:00', 'done', 'Oliver Grant', 'Impressions report, photo sets, show clips, and invoice proof uploaded.', '#831843', 3, '2025-12-18T14:00:00Z', '2026-02-28T18:00:00Z', '', '00000000-0000-7000-8000-000000000002'),
    ('00000000-0000-7000-8000-000000000738', '00000000-0000-7000-8000-000000000105', 'PII purge checklist', 'task', '2026-04-24T09:00:00-04:00', '2026-04-29T17:00:00-04:00', 'blocked', 'Sam Rivera', 'Needs approval before deleting private travel and accommodation records.', '#DC2626', 4, '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z', '', '00000000-0000-7000-8000-000000000020'),
    ('00000000-0000-7000-8000-000000000739', '00000000-0000-7000-8000-000000000106', 'Dataset workshop delivery', 'task', '2026-01-21T10:00:00+01:00', '2026-01-22T16:00:00+01:00', 'done', 'Sofia Marin', 'Municipal data exercises completed and templates exported.', '#1D4ED8', 1, '2025-11-20T13:00:00Z', '2026-01-22T17:00:00Z', '', '00000000-0000-7000-8000-000000000005'),
    ('00000000-0000-7000-8000-000000000740', '00000000-0000-7000-8000-000000000106', 'Accessibility audit clinic', 'milestone', '2026-01-22T14:00:00+01:00', '2026-01-22T14:00:00+01:00', 'done', 'Fatima Al-Sayed', 'Public-service teams reviewed website and data portal barriers.', '#14B8A6', 2, '2025-11-20T13:00:00Z', '2026-01-22T16:00:00Z', '', '00000000-0000-7000-8000-000000000016'),
    ('00000000-0000-7000-8000-000000000741', '00000000-0000-7000-8000-000000000106', 'Grant reporting export', 'asset', '2026-01-25T09:00:00+01:00', '2026-02-02T12:00:00+01:00', 'done', 'Priya Nair', 'Budget, demographics, supplier, and impact files stored for grant closeout.', '#0F766E', 3, '2025-11-20T13:00:00Z', '2026-02-02T12:00:00Z', '', '00000000-0000-7000-8000-000000000003'),
    ('00000000-0000-7000-8000-000000000742', '00000000-0000-7000-8000-000000000106', 'Recording release review', 'task', '2026-01-26T09:00:00+01:00', '2026-01-30T17:00:00+01:00', 'done', 'Liam Connor', 'Recordings tagged public, partner-only, or internal training.', '#64748B', 4, '2025-11-20T13:00:00Z', '2026-01-30T18:00:00Z', '', '00000000-0000-7000-8000-000000000006'),
    ('00000000-0000-7000-8000-000000000743', '00000000-0000-7000-8000-000000000107', 'Workbook outline approved', 'milestone', '2026-09-01T10:00:00Z', '2026-09-01T10:00:00Z', 'done', 'Oliver Grant', 'Learning objectives, exercises, and completion rubric approved.', '#F97316', 1, '2026-04-18T16:20:00Z', '2026-04-29T16:00:00Z', '', '00000000-0000-7000-8000-000000000002'),
    ('00000000-0000-7000-8000-000000000744', '00000000-0000-7000-8000-000000000107', 'Remote classroom platform build', 'task', '2026-10-12T10:00:00Z', '2026-10-23T18:00:00Z', 'planned', 'Kenji Tanaka', 'Configure registration, captions, breakouts, support queue, recordings, and analytics.', '#4338CA', 2, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z', '', '00000000-0000-7000-8000-000000000010'),
    ('00000000-0000-7000-8000-000000000745', '00000000-0000-7000-8000-000000000107', 'Facilitator dry run', 'task', '2026-10-28T12:00:00Z', '2026-10-28T18:00:00Z', 'planned', 'Maya Chen', 'Practice emergency handoff, time-boxed scenarios, and learner support routing.', '#1D4ED8', 3, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z', '00000000-0000-7000-8000-000000000744', '00000000-0000-7000-8000-000000000001'),
    ('00000000-0000-7000-8000-000000000746', '00000000-0000-7000-8000-000000000107', 'Regional watch-room staffing', 'task', '2026-10-29T09:00:00Z', '2026-11-03T18:00:00Z', 'in_progress', 'Grace Kim', 'Confirm Americas, EMEA, and APAC hosts plus backup moderators.', '#14B8A6', 4, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z', '', '00000000-0000-7000-8000-000000000011'),
    ('00000000-0000-7000-8000-000000000747', '00000000-0000-7000-8000-000000000107', 'Backup channel test', 'task', '2026-11-02T14:00:00Z', '2026-11-02T16:00:00Z', 'blocked', 'Lucas Brown', 'Waiting for phone bridge provisioning from vendor.', '#DC2626', 5, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z', '', '00000000-0000-7000-8000-000000000014'),
    ('00000000-0000-7000-8000-000000000748', '00000000-0000-7000-8000-000000000107', 'Masterclass live', 'milestone', '2026-11-04T10:00:00Z', '2026-11-04T10:00:00Z', 'planned', 'Oliver Grant', 'Main digital room opens with global support active.', '#F97316', 6, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z', '00000000-0000-7000-8000-000000000745', '00000000-0000-7000-8000-000000000002');

-- === social publishing queue ===
INSERT INTO social_posts (id, event_id, platform, title, body, status, position, created_at, updated_at) VALUES
    ('00000000-0000-7000-8000-000000000801', '00000000-0000-7000-8000-000000000101', 'linkedin', 'Executive launch agenda is live', 'The Global Product Launch Summit agenda is now live with product reveals, customer proof, and partner showcases across three days in San Francisco.', 'posted', 1, '2026-04-10T12:00:00Z', '2026-04-29T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000802', '00000000-0000-7000-8000-000000000101', 'x', 'Keynote teaser', 'A new product chapter begins on June 16. Follow the launch stream for keynote highlights, demos, and customer stories.', 'ready', 2, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000803', '00000000-0000-7000-8000-000000000101', 'instagram', 'Behind the scenes at Pier 27', 'Venue walkthrough, registration mockups, badge wall tests, and sponsor booth previews from the summit build.', 'draft', 3, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000804', '00000000-0000-7000-8000-000000000101', 'linkedin', 'Partner showcase announcement', 'Our partner showcase will feature implementation stories from global teams using the launch platform in production environments.', 'ready', 4, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000805', '00000000-0000-7000-8000-000000000101', 'youtube', 'Livestream reminder', 'Subscribe for the June 16 livestream and set a reminder for the opening keynote and demo session.', 'draft', 5, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000806', '00000000-0000-7000-8000-000000000101', 'email', 'Know before you go', 'Travel, check-in, accessibility services, venue entry, and evening program details for registered attendees.', 'ready', 6, '2026-04-10T12:00:00Z', '2026-04-30T08:20:00Z'),
    ('00000000-0000-7000-8000-000000000807', '00000000-0000-7000-8000-000000000102', 'linkedin', 'Climate finance leaders convene in London', 'The Climate Finance Forum brings together institutional capital, public finance, and climate-risk experts for practical transition planning.', 'draft', 1, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000808', '00000000-0000-7000-8000-000000000102', 'x', 'Roundtable applications opening', 'Applications for closed-door investor roundtables open soon. Capacity is limited to keep discussion practical and candid.', 'ready', 2, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000809', '00000000-0000-7000-8000-000000000102', 'email', 'Delegate save the date', 'Save September 8 and 9 for two days of transition finance, risk governance, and sector-specific capital planning.', 'posted', 3, '2026-04-12T09:30:00Z', '2026-04-28T10:00:00Z'),
    ('00000000-0000-7000-8000-000000000810', '00000000-0000-7000-8000-000000000102', 'linkedin', 'Sponsor participation guide', 'A concise guide for responsible sponsor messaging, data claims, and delegate value at the forum.', 'draft', 4, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000811', '00000000-0000-7000-8000-000000000102', 'mastodon', 'Hybrid attendance notes', 'Remote participation will include moderated Q and A, captioned plenaries, and session summaries for registered delegates.', 'draft', 5, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000812', '00000000-0000-7000-8000-000000000102', 'press', 'Media policy preview', 'The forum will publish a session-by-session media access policy before accreditation opens.', 'ready', 6, '2026-04-12T09:30:00Z', '2026-04-30T08:25:00Z'),
    ('00000000-0000-7000-8000-000000000813', '00000000-0000-7000-8000-000000000103', 'linkedin', 'APAC Developer Week workshop catalog', 'Browse hands-on labs covering platform engineering, secure APIs, observability, AI tooling, and open-source maintenance.', 'posted', 1, '2026-04-14T07:45:00Z', '2026-04-25T11:00:00Z'),
    ('00000000-0000-7000-8000-000000000814', '00000000-0000-7000-8000-000000000103', 'x', 'Call for proposals closing soon', 'Last chance to submit a technical talk, lab, or maintainer session for Asia Pacific Developer Week.', 'ready', 2, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000815', '00000000-0000-7000-8000-000000000103', 'instagram', 'Community lounge preview', 'A first look at the maintainer lounge, hallway-track boards, and mentor office-hours space.', 'draft', 3, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000816', '00000000-0000-7000-8000-000000000103', 'youtube', 'Workshop setup guide', 'Prepare your local environment before arriving so you can spend more time building in the labs.', 'draft', 4, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000817', '00000000-0000-7000-8000-000000000103', 'newsletter', 'Maintainer interview series', 'Meet the open-source maintainers leading conversations on governance, funding, release quality, and community health.', 'ready', 5, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000818', '00000000-0000-7000-8000-000000000103', 'linkedin', 'Student and early-career guide', 'How to use mentor hours, project tables, hallway boards, and scholarship programming during Developer Week.', 'draft', 6, '2026-04-14T07:45:00Z', '2026-04-30T08:30:00Z'),
    ('00000000-0000-7000-8000-000000000819', '00000000-0000-7000-8000-000000000104', 'linkedin', 'Humanitarian Logistics Expo recap', 'Three days of practical response planning, field-connectivity demos, and medical supply-chain collaboration in Nairobi.', 'posted', 1, '2026-01-08T10:00:00Z', '2026-03-14T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000820', '00000000-0000-7000-8000-000000000104', 'email', 'After-action survey reminder', 'Share supplier feedback, field-readiness notes, and partner follow-up requests before the survey closes.', 'posted', 2, '2026-01-08T10:00:00Z', '2026-03-17T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000821', '00000000-0000-7000-8000-000000000104', 'press', 'Emergency response tabletop outcomes', 'Observer notes from the tabletop exercise are now available to registered partner organizations.', 'ready', 3, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z'),
    ('00000000-0000-7000-8000-000000000822', '00000000-0000-7000-8000-000000000104', 'linkedin', 'Supplier recognition post', 'Thanking logistics, translation, venue, and medical suppliers who supported the field-readiness program.', 'draft', 4, '2026-01-08T10:00:00Z', '2026-04-30T08:40:00Z'),
    ('00000000-0000-7000-8000-000000000823', '00000000-0000-7000-8000-000000000105', 'instagram', 'Awards night highlights', 'Finalist arrivals, stage moments, and winner portraits from the Creator Economy Awards.', 'posted', 1, '2025-12-18T14:00:00Z', '2026-02-21T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000824', '00000000-0000-7000-8000-000000000105', 'youtube', 'Winner acceptance playlist', 'A curated playlist of winner speeches and short-form clips for sponsor reporting.', 'posted', 2, '2025-12-18T14:00:00Z', '2026-02-24T09:00:00Z'),
    ('00000000-0000-7000-8000-000000000825', '00000000-0000-7000-8000-000000000105', 'email', 'Sponsor closeout package', 'Final impressions, stage photography, branded clips, and invoice references are ready for sponsor review.', 'ready', 3, '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000826', '00000000-0000-7000-8000-000000000105', 'linkedin', 'Final archive notice', 'The event workspace is pending destruction after export and retention checks are complete.', 'draft', 4, '2025-12-18T14:00:00Z', '2026-04-28T15:00:00Z'),
    ('00000000-0000-7000-8000-000000000827', '00000000-0000-7000-8000-000000000106', 'linkedin', 'Civic Data Lab impact recap', 'Archived recap covering municipal prototypes, accessibility improvements, public datasets, and partner commitments.', 'posted', 1, '2025-11-20T13:00:00Z', '2026-02-02T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000828', '00000000-0000-7000-8000-000000000106', 'newsletter', 'Reusable workshop materials', 'Facilitator templates, anonymized datasets, and public-service worksheets are available for partner teams.', 'posted', 2, '2025-11-20T13:00:00Z', '2026-01-30T11:00:00Z'),
    ('00000000-0000-7000-8000-000000000829', '00000000-0000-7000-8000-000000000106', 'press', 'Grant reporting complete', 'The lab has completed grant reporting with budget, demographic, supplier, and evaluation summaries.', 'posted', 3, '2025-11-20T13:00:00Z', '2026-02-02T12:00:00Z'),
    ('00000000-0000-7000-8000-000000000830', '00000000-0000-7000-8000-000000000107', 'linkedin', 'Remote Operations Masterclass announced', 'A practical digital-first training day for distributed teams managing incidents, handoffs, and support across time zones.', 'ready', 1, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000831', '00000000-0000-7000-8000-000000000107', 'x', 'Workbook preview', 'Participants will leave with templates for incident notes, shift handoffs, escalation trees, and async status updates.', 'draft', 2, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z'),
    ('00000000-0000-7000-8000-000000000832', '00000000-0000-7000-8000-000000000107', 'email', 'Registration interest list', 'Join the interest list for remote operations training and receive the agenda when registration opens.', 'posted', 3, '2026-04-18T16:20:00Z', '2026-04-29T16:00:00Z'),
    ('00000000-0000-7000-8000-000000000833', '00000000-0000-7000-8000-000000000107', 'youtube', 'Platform walkthrough teaser', 'A short preview of the virtual classroom, breakout flow, support queue, and backup channels.', 'draft', 4, '2026-04-18T16:20:00Z', '2026-04-30T08:35:00Z');

COMMIT;

-- quick smoke-check for local CLI runs
SELECT 'users' AS table_name, COUNT(*) AS rows FROM users
UNION ALL SELECT 'magic_tokens', COUNT(*) FROM magic_tokens
UNION ALL SELECT 'magic_link_outbox', COUNT(*) FROM magic_link_outbox
UNION ALL SELECT 'events', COUNT(*) FROM events
UNION ALL SELECT 'event_memberships', COUNT(*) FROM event_memberships
UNION ALL SELECT 'event_branding', COUNT(*) FROM event_branding
UNION ALL SELECT 'planner_items', COUNT(*) FROM planner_items
UNION ALL SELECT 'planner_timeline_items', COUNT(*) FROM planner_timeline_items
UNION ALL SELECT 'social_posts', COUNT(*) FROM social_posts;
