CREATE TYPE Platform AS ENUM ('Dev', 'Steam');
CREATE TYPE Privilege AS ENUM ('None', 'Manager');

CREATE TABLE accounts (
    id          bigserial primary key,
    created     timestamp not null default now(),
    platform    Platform  not null,
    platform_id bigint    not null,

    privilege Privilege not null default 'None'
);