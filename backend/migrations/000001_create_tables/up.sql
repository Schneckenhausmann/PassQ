CREATE TABLE users (
    id UUID PRIMARY KEY,
    username VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    salt VARCHAR NOT NULL,
    mfa_secret VARCHAR
);

CREATE TABLE folders (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL REFERENCES users(id),
    parent_folder_id UUID REFERENCES folders(id),
    name VARCHAR NOT NULL,
    CONSTRAINT folder_user_check CHECK (parent_folder_id IS DISTINCT FROM id)
);

CREATE TABLE passwords (
    id UUID PRIMARY KEY,
    folder_id UUID REFERENCES folders(id),
    website VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    encrypted_password BYTEA NOT NULL
);

CREATE TABLE shares (
    id UUID PRIMARY KEY,
    password_id UUID REFERENCES passwords(id),
    folder_id UUID REFERENCES folders(id),
    user_id UUID NOT NULL REFERENCES users(id)
);