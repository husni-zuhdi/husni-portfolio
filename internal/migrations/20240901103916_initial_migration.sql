-- Add migration script here
CREATE TABLE IF NOT EXISTS blogs (
    id          TEXT    PRIMARY KEY NOT NULL,
    name        TEXT                NOT NULL,
    source      TEXT                NOT NULL,
    filename    TEXT                NOT NULL,
    body        TEXT                NOT NULL
);
