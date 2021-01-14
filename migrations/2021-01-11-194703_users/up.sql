-- Your SQL goes here
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE users (
    user_id uuid PRIMARY KEY DEFAULT uuid_generate_v4() NOT NULL UNIQUE,
    username text NOT NULL UNIQUE,
    password text NOT NULL
);