#!/bin/sh

DATA_ROOT="$(dirname "$(realpath "$0")")"

cd $DATA_ROOT/tools/validator
go run main.go -root="$DATA_ROOT"
