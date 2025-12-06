create table item (
    id bigint not null,
    character_id bigint not null,
    data_id integer not null,
    count integer not null,
    level smallint not null,
    is_bound boolean not null default false,
    attributes jsonb,

    primary key (id, character_id),
    foreign key (character_id) references character (id) on delete cascade
);
