#!/usr/bin/env bash
set -e

v="$1"
if [ -z "$v" ]; then
    echo "usage: $0 <pg_version>"
    exit 1
fi

name="pg${v}"

if cargo pgrx info pg-config "$name" >/dev/null 2>&1; then
    echo "${name} already initialized"
else
    cargo pgrx init "--${name}" download
fi

PG_CONFIG_PATH=$(cargo pgrx info pg-config "$v")

cargo pgrx package \
    --pg-config "$PG_CONFIG_PATH" \
    --features "pg$v" \
    --no-default-features

base="$(pwd)/target/release/sqlgen-pg${v}"

SQLGEN_SO_PATH="$(find . -type f -name 'sqlgen.so' -print -quit)"
SQLGEN_CONTROL_PATH="$(find . -type f -name 'sqlgen.control' -print -quit)"
SQLGEN_SQL_PATH="$(find . -type f -name 'sqlgen--*.sql' -print -quit)"

if [ -n "$SQLGEN_SO_PATH" ]; then
    SQLGEN_SO_PATH="$(readlink -f "$SQLGEN_SO_PATH")"
fi

if [ -n "$SQLGEN_CONTROL_PATH" ]; then
    SQLGEN_CONTROL_PATH="$(readlink -f "$SQLGEN_CONTROL_PATH")"
fi

if [ -n "$SQLGEN_SQL_PATH" ]; then
    SQLGEN_SQL_PATH="$(readlink -f "$SQLGEN_SQL_PATH")"
fi

echo "$SQLGEN_SO_PATH"
echo "$SQLGEN_CONTROL_PATH"
echo "$SQLGEN_SQL_PATH"