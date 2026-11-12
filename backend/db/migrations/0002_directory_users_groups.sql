CREATE TABLE users (
    uid TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    email TEXT NOT NULL DEFAULT '',
    password_hash TEXT NOT NULL
);
CREATE TABLE groups (
    gid TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT ''
);
CREATE TABLE group_members (
    gid TEXT NOT NULL REFERENCES groups(gid) ON DELETE CASCADE,
    uid TEXT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    PRIMARY KEY (gid, uid)
);
