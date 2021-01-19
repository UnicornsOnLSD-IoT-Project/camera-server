-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE camera_tokens (
    camera_token uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL UNIQUE,
    camera_id uuid NOT NULL,
    CONSTRAINT fk_camera_id
        FOREIGN KEY (camera_id)
            REFERENCES cameras (camera_id)
            ON DELETE CASCADE
)