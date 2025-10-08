create table dev_account (
    id varchar(16) not null,
    account_id bigint not null,
    created_at timestamptz default now() not null,

    primary key (id),
    foreign key (account_id) references account (id) on delete cascade
);
