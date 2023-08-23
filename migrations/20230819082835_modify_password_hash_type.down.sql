-- Add down migration script here
alter table users alter column password_hash type bytea;
