#!/bin/bash

docker compose \
    --env-file .env \
    --env-file dev.env \
    -f compose.yaml \
    -f compose.dev.yaml \
    "$@"
