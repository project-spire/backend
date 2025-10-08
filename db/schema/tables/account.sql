create table account (
    id bigint not null,
    created_at timestamptz default now() not null,

    primary key (id)
);
