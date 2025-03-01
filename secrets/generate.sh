#!/bin/bash

openssl req -x509 -newkey rsa:4096 -keyout game-key.pem -out game-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

openssl req -x509 -newkey rsa:4096 -keyout lobby-key.pem -out lobby-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

openssl rand -base64 64 > auth-key.txt