create table character_talent (
    character_id bigint not null,
    data_id integer not null,
    is_active boolean not null default true,
    level smallint not null default 0,
    exp integer not null default 0,

    primary key (character_id, data_id),
    foreign key (character_id) references character (id) on delete cascade
);
