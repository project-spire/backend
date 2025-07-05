CREATE TYPE EquipmentItemType AS ENUM (
    'Weapon',
    'Armor'
);

CREATE TABLE equipment_items (
    item_id bigint primary key references item(id),
    type    EquipmentItemType not null,
    created timestamp not null default now(),

    durability integer not null default 0
);
