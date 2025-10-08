#!/bin/bash

docker compose \
    --env-file .env \
    --env-file local.env \
    -f compose.yaml \
    -f compose.local.yaml \
    "$@"
