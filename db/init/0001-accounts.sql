CREATE TABLE accounts
(
    id serial primary key,
    created timestamp default now()
);