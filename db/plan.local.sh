#!/bin/bash

set -e

source ../.env
source ../local.env

password=$(cat ../secrets/db-password.key)

pgschema plan \
    --host "$SPIRE_DB_HOST" \
    --port "$SPIRE_DB_PORT" \
    --db "$SPIRE_DB_NAME" \
    --user "$SPIRE_DB_USER" \
    --password "$password" \
    --file schema/main.sql \
    --output-json plan.json \
    --output-human plan.txt

cat plan.txt
