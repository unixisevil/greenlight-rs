CREATE EXTENSION IF NOT EXISTS citext;

CREATE TABLE IF NOT EXISTS movies (
    id bigserial PRIMARY KEY,  
    created_at timestamp(0) with time zone NOT NULL DEFAULT NOW(),
    title text NOT NULL,
    year integer NOT NULL,
    runtime integer NOT NULL,
    genres text[] NOT NULL,
    version integer NOT NULL DEFAULT 1
);

ALTER TABLE movies ADD CONSTRAINT movies_runtime_check CHECK (runtime >= 0);
ALTER TABLE movies ADD CONSTRAINT movies_year_check CHECK (year BETWEEN 1888 AND date_part('year', now()));
ALTER TABLE movies ADD CONSTRAINT genres_length_check CHECK (array_length(genres, 1) BETWEEN 1 AND 5);

CREATE INDEX IF NOT EXISTS movies_title_idx ON movies USING GIN (to_tsvector('simple', title));
CREATE INDEX IF NOT EXISTS movies_genres_idx ON movies USING GIN (genres);


CREATE TABLE IF NOT EXISTS users (
    id bigserial PRIMARY KEY,
    created_at timestamp(0) with time zone NOT NULL DEFAULT NOW(),
    name text NOT NULL,
    email citext UNIQUE NOT NULL,
    password_hash text NOT NULL,
    activated bool NOT NULL,
    version integer NOT NULL DEFAULT 1
);

CREATE TABLE IF NOT EXISTS tokens (
    hash bytea PRIMARY KEY,
    user_id bigint NOT NULL REFERENCES users ON DELETE CASCADE,
    expiry timestamp(0) with time zone NOT NULL,
    scope text NOT NULL
);

CREATE TABLE IF NOT EXISTS permissions (
    id bigserial PRIMARY KEY,
    code text NOT NULL
);

CREATE TABLE IF NOT EXISTS users_permissions (
    user_id bigint NOT NULL REFERENCES users ON DELETE CASCADE,
    permission_id bigint NOT NULL REFERENCES permissions ON DELETE CASCADE,
    PRIMARY KEY (user_id, permission_id)
);

-- add the two permissions to the table
INSERT INTO permissions (code) VALUES ('movies:read'), ('movies:write');

-- seed user alice and bob
insert into users (name, email, password_hash, activated) values 
('alice', 'alice@example.com', '$argon2id$v=19$m=15000,t=2,p=1$cB5dpwlRXNmG4gZ3Wd0brQ$XE1vZSzgGs1lJeWt7ha3C+3ujyBDh/cbnJtkP0hbMn8',  true),
('bob', 'bob@example.com', '$argon2id$v=19$m=15000,t=2,p=1$fjJQedxhqZQ2gYK90Qraeg$BiAA5bHU+dSHIDJpNagF1kwPcKjjTznUmVkyzouviRU', true);

-- give alice and bob 'movies:read' permission
insert into users_permissions select id, (select id from permissions where code = 'movies:read') from users;

-- give alice 'movies:write' permission
insert into users_permissions
values (
	(select id from users where email = 'alice@example.com'),
	(select id from permissions where code = 'movies:write')
);
