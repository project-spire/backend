CREATE TABLE dev_accounts (
    id         varchar(16) primary key,
    account_id bigint      references accounts(id)
);