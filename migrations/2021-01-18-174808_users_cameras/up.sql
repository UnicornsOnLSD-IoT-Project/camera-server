-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE users_cameras (
    users_cameras_id SERIAL PRIMARY KEY,
    camera_id uuid NOT NULL,
    user_id uuid NOT NULL,
    CONSTRAINT fk_camera_id
        FOREIGN KEY (camera_id)
            REFERENCES cameras (camera_id)
            ON DELETE CASCADE,
    CONSTRAINT fk_user_id
        FOREIGN KEY (user_id)
            REFERENCES users (user_id)
            ON DELETE CASCADE
);