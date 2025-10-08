#!/bin/bash

set -e

source ../.env
source ../local.env

password=$(cat ../secrets/db-password.key)

pgschema apply \
    --host "$SPIRE_DB_HOST" \
    --port "$SPIRE_DB_PORT" \
    --db "$SPIRE_DB_NAME" \
    --user "$SPIRE_DB_USER" \
    --password "$password" \
    --plan plan.json
