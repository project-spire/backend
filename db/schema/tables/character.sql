create table character (
    id bigint not null,
    account_id bigint not null,
    created_at timestamptz not null default now(),

    -- Identity
    name varchar(16) not null,
    race Race not null,

    -- Growth
    level smallint not null default 0,
    exp bigint not null default 0,
    karma bigint not null default 0,

    -- World
    "location" location not null default (0, 0),
    position vector3 not null default row(0, 0, 0),

    -- Resource
    health bigint not null default 0,
    mana bigint,

    -- Asset
    gold bigint not null default 0,

    primary key (id),
    foreign key (account_id) references account (id) on delete cascade
);
