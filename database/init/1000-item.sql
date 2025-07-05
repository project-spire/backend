CREATE TYPE ItemType AS ENUM (
    'Asset',
    'Equipment',
    'Consumable',
    'Material'
);

CREATE TABLE item (
    id   bigserial primary key,
    type ItemType not null
);
