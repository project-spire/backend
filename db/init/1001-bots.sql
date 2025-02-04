CREATE TABLE bots
(
    bot_id     INT PRIMARY KEY,
    account_id INT NOT NULL,
    created    TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP

    FOREIGN KEY (account_id) REFERENCES accounts (account_id)
);