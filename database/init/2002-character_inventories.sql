CREATE TABLE character_inventories (
    character_id bigint not null references characters(id),
    item_id      bigint not null references items(id),

    primary key(character_id, item_id)
);
