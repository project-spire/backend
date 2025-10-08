create table dev_account (
    id varchar(16) primary key,
    account_id bigint references account(id) on delete cascade,
    created_at timestamptz default now() not null
);
