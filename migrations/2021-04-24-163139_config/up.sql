-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE configs (
    camera_id uuid PRIMARY KEY NOT NULL UNIQUE,
    interval smallint DEFAULT 60 NOT NULL,
    CONSTRAINT fk_camera_id
        FOREIGN KEY (camera_id)
            REFERENCES cameras (camera_id)
            ON DELETE CASCADE
)