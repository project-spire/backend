#!/bin/bash

set -e

source ../.env
source ../local.env

password=$(cat ../secrets/db-password.key)
url="postgres://${SPIRE_DB_USER}:${password}@${SPIRE_DB_HOST}:${SPIRE_DB_PORT}/${SPIRE_DB_NAME}?search_path=public&sslmode=disable"

atlas version
atlas schema apply \
    --env dev \
    --url "$url"
