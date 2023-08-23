-- Add up migration script here

alter table users alter column password_hash type text;
