#!/bin/bash

set -e

source ../.env
source ../dev.env

password=$(cat ../secrets/db-password.key)
url="postgres://${SPIRE_DB_USER}:${password}@${SPIRE_DB_HOST}:${SPIRE_DB_PORT}/${SPIRE_DB_NAME}?search_path=public&sslmode=disable"

cd ..
cargo sqlx prepare \
    --workspace \
    --database-url "$url"
