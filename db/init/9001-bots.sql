CREATE TABLE bots (
    id         bigint primary key,
    account_id bigint references accounts(id)
);