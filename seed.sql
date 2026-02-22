-- dummy data for fe/be development
-- run with sqlite3 data.db < seed.sql

CREATE TABLE IF NOT EXISTS users (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    surname TEXT NOT NULL,
    email TEXT NOT NULL,
    phone TEXT NOT NULL,
    role TEXT NOT NULL,
    status TEXT NOT NULL
);

-- clear past dev data if present
DELETE FROM users WHERE email IN ('alice@circa.local', 'bob@circa.local');

INSERT INTO users (id, name, surname, email, phone, role, status)
VALUES ('019c8555-7a32-719a-bbfc-289d208c2996', 'Alice', 'Lovelace', 'alice@circa.local', '+1-023-456-789', 'admin', 'active');

INSERT INTO users (id, name, surname, email, phone, role, status)
VALUES ('019c8555-7a32-7972-8961-f2c2b29ebd22', 'Bob', 'Birkenstock', 'bob@circa.local', '+1-321-654-987', 'organizer', 'inactive');

-- check if added correctly :3
SELECT id, name, email, role, status FROM users;
