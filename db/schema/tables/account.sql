create table account (
    id bigint primary key,
    created_at timestamptz default now() not null
);
