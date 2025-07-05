CREATE TABLE character_inventory (
    character_id bigint not null references character(id),
    item_id      bigint not null references item(id),

    primary key(character_id, item_id)
);
