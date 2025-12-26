#!/bin/bash

set -e

source ../.env
source ../local.env

password=$(cat ../secrets/db-password.key)
url="postgres://${SPIRE_DB_USER}:${password}@${SPIRE_DB_HOST}:${SPIRE_DB_PORT}/${SPIRE_DB_NAME}"

diesel print-schema \
    --database-url "$url" \
    > ../data/src/schema.rs
