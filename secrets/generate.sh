#!/bin/bash

# Game Server Certfication
openssl req -x509 -newkey rsa:4096 -keyout game-server-key.pem -out game-server-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

# Lobby Server Certfication
openssl req -x509 -newkey rsa:4096 -keyout lobby-server-key.pem -out lobby-server-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

# DB Certfication
openssl req -x509 -newkey rsa:4096 -keyout db-key.pem -out db-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=db" \
    -addext "subjectAltName=DNS:localhost,DNS:db"
chmod +r db-key.pem

# 256 bit authentication token key
openssl rand -base64 32 | head -c -1 > token.key
