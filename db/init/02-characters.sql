CREATE TABLE characters
(
    character_id   INT AUTO_INCREMENT PRIMARY KEY,
    account_id     INT,

    character_name VARCHAR(30) UNIQUE NOT NULL CHECK (character_name REGEXP '^[a-zA-Z0-9_]{6,30}$'),
    created        TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (account_id) REFERENCES accounts (account_id)
);
CREATE INDEX idx_characters_account_id ON characters (account_id);