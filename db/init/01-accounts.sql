CREATE TABLE accounts
(
    account_id        INT AUTO_INCREMENT PRIMARY KEY,
    -- username  VARCHAR(30) UNIQUE NOT NULL CHECK (username REGEXP '^[a-zA-Z0-9_]{6,30}$'),
    -- platform  VARCHAR(8)         NOT NULL CHECK (platform IN ('TEST', 'STEAM')),
    -- status    VARCHAR(8)         NOT NULL DEFAULT 'ACTIVE' CHECK (status IN ('ACTIVE', 'INACTIVE', 'BLOCKED')),
    -- privilege VARCHAR(8)         NULL CHECK (privilege IN (NULL, 'MANAGER', 'ADMIN')),
    created   TIMESTAMP          NOT NULL DEFAULT CURRENT_TIMESTAMP
);