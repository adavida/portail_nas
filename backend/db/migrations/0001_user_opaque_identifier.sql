CREATE TABLE IF NOT EXISTS user_opaque_identifier (
    client_id TEXT NOT NULL,
    username TEXT NOT NULL,
    identifier TEXT NOT NULL,
    PRIMARY KEY (client_id, username)
);
