const fs = require('fs');

const user = process.env.SPIRE_DB_USER;
const passwordFile = process.env.SPIRE_DB_PASSWORD_FILE;
const database = process.env.SPIRE_DB_USER;

if (!user || !passwordFile || !database) {
    print('Missing required environment variables');
    quit(1);
}

let password;
try {
    password = fs.readFileSync(passwordFile, 'utf8').trim();
} catch (err) {
    print(`Error reading password file: ${err}`);
    quit(1);
}

db.createUser({
    user: user,
    pwd: password,
    roles: [{ role: 'readWrite', db: database }]
});