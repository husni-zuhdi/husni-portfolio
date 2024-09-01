-- Add migration script here
CREATE TABLE IF NOT EXISTS blogs (
    id          INTEGER PRIMARY KEY NOT NULL,
    name        TEXT                NOT NULL,
    source      TEXT                NOT NULL,
    filename    TEXT                NOT NULL,
    body        TEXT                NOT NULL
);

CREATE TABLE IF NOT EXISTS github_trees (
    id          INTEGER PRIMARY KEY NOT NULL,
    tree_path   TEXT                NOT NULL,
    tree_mode   TEXT                NOT NULL,
    tree_type   TEXT                NOT NULL,
    sha         TEXT                NOT NULL,
    url         TEXT                NOT NULL
);
