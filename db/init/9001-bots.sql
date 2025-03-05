CREATE TABLE bots
(
    id bigserial primary key,
    account_id bigint references accounts(id)
);