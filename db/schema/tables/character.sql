create table character (
    id bigint not null,
    account_id bigint not null,
    created_at timestamptz default now() not null,
    name varchar(16) not null,
    race Race not null,

    primary key (id),
    foreign key (account_id) references account (id) on delete cascade
);
