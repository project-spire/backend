CREATE TYPE Race AS ENUM (
    'Human',
    'Barbarian',
    'Elf'
);

CREATE TABLE characters (
    id         bigserial primary key,
    account_id bigint    references accounts(id),
    created    timestamp not null default now(),

    name varchar(32) unique not null,
    race Race               not null
);