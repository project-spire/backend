CREATE TABLE dev_account (
    id         varchar(16) primary key,
    account_id bigint      references account(id)
);