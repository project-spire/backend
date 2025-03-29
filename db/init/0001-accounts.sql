CREATE TYPE Privilege AS ENUM ('None', 'Manager');

CREATE TABLE accounts (
    id        bigserial primary key,
    created   timestamp default now(),
    privilege Privilege not null default 'None'
);