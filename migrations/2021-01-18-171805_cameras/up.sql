-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE cameras (
    camera_id uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL UNIQUE,
    name text NOT NULL
);