#!/usr/bin/env bash
set -e

VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/version *= *"\(.*\)"/\1/')
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

echo "cargo pgrx package \
    --pg-config "$PG_CONFIG_PATH" \
    --features "pg$v" \
    --no-default-features"

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

EXTENSION_DIR=./target/package-${v}/usr/share/postgresql/${v}/extension/
LIB_DIR=./target/package-${v}/usr/lib/postgresql/${v}/lib/

rm -rf ${EXTENSION_DIR}
rm -rf ${LIB_DIR}

mkdir -p ${EXTENSION_DIR}
mkdir -p ${LIB_DIR}

cp ${SQLGEN_CONTROL_PATH} ${EXTENSION_DIR}
cp ${SQLGEN_SQL_PATH} ${EXTENSION_DIR}
cp ${SQLGEN_SO_PATH} ${LIB_DIR}


fpm \
    -s dir \
    -t deb \
    -n pg_sqlgen\
    -v ${VERSION} \
    -C target/package-18 \
    .

# TODO have it run for every major version