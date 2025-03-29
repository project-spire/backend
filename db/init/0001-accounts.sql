CREATE TABLE accounts (
    id      bigserial primary key,
    created timestamp default now()
);