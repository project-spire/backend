CREATE TABLE characters
(
    id serial primary key,
    account_id integer references accounts(id),
    created timestamp default now()
);