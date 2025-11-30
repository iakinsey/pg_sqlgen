#!/usr/bin/env bash
set -e

VERSIONS=(14 15 16 17 18)

for v in "${VERSIONS[@]}"; do
    initialized=1
    name="pg${v}"

    if cargo pgrx info pg-config "$name" >/dev/null 2>&1; then
        initialized=0
    fi

    if [ "$initialized" -ne 0 ]; then
        cargo pgrx init "--${name}" download
    else
        echo "${name} already initialized"
    fi

    PG_CONFIG_PATH=$(cargo pgrx info pg-config $v)

    cargo pgrx package \
        --pg-config "$PG_CONFIG_PATH" \
        --features "pg$v" \
        --no-default-features
done
