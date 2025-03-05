CREATE TABLE characters
(
    id bigserial primary key,
    account_id bigint references accounts(id),
    name varchar(24) unique,
    created timestamp default now()
);