#!/bin/bash

# Game Server Certificate
openssl req -x509 -newkey rsa:4096 -keyout game-server-key.pem -out game-server-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=game.spire.dev" \
    -addext "subjectAltName=DNS:game.spire.dev"

# Lobby Server Certificate
openssl req -x509 -newkey rsa:4096 -keyout lobby-server-key.pem -out lobby-server-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=lobby.spire.dev" \
    -addext "subjectAltName=DNS:lobby.spire.dev"

# DB Certificate
openssl req -x509 -newkey rsa:4096 -keyout db-key.pem -out db-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=db.spire.dev" \
    -addext "subjectAltName=DNS:db.spire.dev"
chmod +r db-key.pem

# Token Key
openssl rand -base64 32 | head -c -1 > token.key

# DB Password
echo "password" | head -c -1 > db-password.key
