-- Add migration script here
create table if not exists todos (
    id text primary key,
    text text not null,
    completed boolean not null default false
);