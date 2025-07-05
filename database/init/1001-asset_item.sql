CREATE TYPE AssetItemType AS ENUM (
    'Gold',
    'Soul'
);

CREATE TABLE asset_item (
    item_id bigint primary key references item(id),
    type    AssetItemType not null,

    quantity integer not null default 0
);
