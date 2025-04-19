CREATE TYPE AssetItemType AS ENUM (
    'Gold',
    'Soul'
);

CREATE TABLE asset_items (
    item_id bigint primary key references items(id),
    type    AssetItemType not null,

    quantity integer not null default 0
);
