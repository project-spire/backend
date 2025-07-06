#!/bin/bash

source ../.env
db_password=$(cat ../secrets/db-password.txt)

atlas schema apply \
    -u "postgres://${SPIRE_DB_USER}:${db_password}@localhost:${SPIRE_DB_PORT}/${SPIRE_DB_NAME}?sslmode=disable"\
    --env "dev"
