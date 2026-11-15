CREATE TABLE attr_labels (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE user_attrs (
    user_id TEXT NOT NULL REFERENCES users(uid) ON DELETE CASCADE,
    label_id INTEGER NOT NULL REFERENCES attr_labels(id) ON DELETE CASCADE,
    value TEXT NOT NULL,
    PRIMARY KEY (user_id, label_id)
);
