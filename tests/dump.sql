PRAGMA foreign_keys=OFF;
BEGIN TRANSACTION;
CREATE TABLE IF NOT EXISTS talks (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            date TEXT NOT NULL,
            media_link TEXT,
            org_name TEXT,
            org_link TEXT);
INSERT INTO talks VALUES(1,'Cloud Computing 101','2022-07-02','','UIN Syarif Hidayatullah Jakarta','');
CREATE TABLE IF NOT EXISTS blogs (
            id INTEGER PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            source TEXT NOT NULL,
            filename TEXT NOT NULL,
            body TEXT NOT NULL, tags TEXT);
INSERT INTO blogs VALUES(1,'test','','',replace('                        Your new blog...\n                    ','\n',char(10)),NULL);
CREATE TABLE IF NOT EXISTS tags (
id INTEGER PRIMARY KEY NOT NULL,
name TEXT NOT NULL
);
INSERT INTO tags VALUES(1,'cloud');
INSERT INTO tags VALUES(2,'k8s');
CREATE TABLE IF NOT EXISTS blog_tag_mapping(
blog_ref INTEGER NOT NULL,
tag_ref INTEGER NOT NULL
);
INSERT INTO blog_tag_mapping VALUES(1,1);
INSERT INTO blog_tag_mapping VALUES(1,2);
CREATE TABLE IF NOT EXISTS users (
                    id TEXT NOT NULL,
                    email TEXT NOT NULL,
                    hashed_password TEXT NOT NULL
                );
CREATE TABLE IF NOT EXISTS sessions (
                    id TEXT NOT NULL,
                    user_id TEXT NOT NULL,
                    token TEXT NOT NULL,
                    expire TEXT NOT NULL
                );
COMMIT;
