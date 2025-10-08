create table character (
    id bigint primary key,
    account_id bigint references account(id) on delete cascade,
    created_at timestamptz default now() not null,

    name varchar(16)
);
