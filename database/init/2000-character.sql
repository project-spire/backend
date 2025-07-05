CREATE TYPE Race AS ENUM (
    'Human',
    'Barbarian',
    'Elf'
);

CREATE TABLE character (
    id         bigserial primary key,
    account_id bigint    references account(id),
    created    timestamp not null default now(),

    name varchar(24) unique not null,
    race Race               not null
);
