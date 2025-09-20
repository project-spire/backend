#!/bin/bash

set -e

# Game Server Certificate
echo "Generating self-signed game server certificate..."
openssl req -quiet -x509 -newkey rsa:4096 \
    -keyout game-server-key.pem -out game-server-cert.pem \
    -sha256 -days 365 -nodes \
    -subj "/CN=game.spire.dev" \
    -addext "subjectAltName=DNS:game.spire.dev" \
    2>/dev/null

# Lobby Server Certificate
echo "Generating self-signed lobby server certificate..."
openssl req -quiet -x509 -newkey rsa:4096 \
    -keyout lobby-server-key.pem -out lobby-server-cert.pem \
    -sha256 -days 365 -nodes \
    -subj "/CN=lobby.spire.dev" \
    -addext "subjectAltName=DNS:lobby.spire.dev" \
    2>/dev/null

# Token Key
openssl rand -base64 32 | head -c -1 > token.key

# Database Password
if [ ! -f db-password.key ]; then
    echo "Generating DB password..."
    echo "password" | head -c -1 > db-password.key
else
    echo "DB password already exist. Skipping..."
fi

# Database Admin Password
if [ ! -f db-admin-password.key ]; then
    echo "Generating DB admin password..."
    echo "password" | head -c -1 > db-admin-password.key
else
    echo "DB admin password already exist. Skipping..."
fi

echo "Generate complete!"
