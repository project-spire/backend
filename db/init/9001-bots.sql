CREATE TABLE bots
(
    id integer primary key,
    account_id integer references accounts(id)
);