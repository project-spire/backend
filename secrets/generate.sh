#!/bin/bash

openssl req -x509 -newkey rsa:4096 -keyout logic-key.pem -out logic-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

openssl req -x509 -newkey rsa:4096 -keyout lobby-key.pem -out lobby-cert.pem \
    -sha256 -days 3650 -nodes \
    -subj "/CN=localhost" \
    -addext "subjectAltName=DNS:localhost"

# 256 bit authentication token key
openssl rand -base64 32 > auth-key.txt