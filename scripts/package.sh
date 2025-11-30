#!/usr/bin/env bash
set -euo pipefail

BUILD_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/version *= *"\(.*\)"/\1/')
VERSIONS=(14 15 16 17 18)

# If first arg is --osx, only build osxpkg, otherwise deb/rpm/pacman
if [[ "${1-}" == "--osx" ]]; then
    PKG_PLATFORMS=(osxpkg)
else
    PKG_PLATFORMS=(deb rpm pacman tar)
fi

for v in "${VERSIONS[@]}"; do
    name="pg${v}"

    if cargo pgrx info pg-config "$name" >/dev/null 2>&1; then
        echo "${name} already initialized"
    else
        cargo pgrx init "--${name}" download
    fi

    PG_CONFIG_PATH=$(cargo pgrx info pg-config "$v")

    cargo pgrx package \
        --pg-config $PG_CONFIG_PATH \
        --features pg$v \
        --no-default-features


    base="$(pwd)/target/release/sqlgen-pg${v}"

    SQLGEN_SO_PATH="$(find ${base} -type f -name 'sqlgen.so' -print -quit)"
    SQLGEN_CONTROL_PATH="$(find ${base} -type f -name 'sqlgen.control' -print -quit)"
    SQLGEN_SQL_PATH="$(find ${base} -type f -name 'sqlgen--*.sql' -print -quit)"

    if [ -n "$SQLGEN_SO_PATH" ]; then
        SQLGEN_SO_PATH="$(readlink -f "$SQLGEN_SO_PATH")"
    fi

    if [ -n "$SQLGEN_CONTROL_PATH" ]; then
        SQLGEN_CONTROL_PATH="$(readlink -f "$SQLGEN_CONTROL_PATH")"
    fi

    if [ -n "$SQLGEN_SQL_PATH" ]; then
        SQLGEN_SQL_PATH="$(readlink -f "$SQLGEN_SQL_PATH")"
    fi

    EXTENSION_DIR=./target/package/usr/share/postgresql/${v}/extension/
    LIB_DIR=./target/package/usr/lib/postgresql/${v}/lib/

    rm -rf "${EXTENSION_DIR}"
    rm -rf "${LIB_DIR}"

    mkdir -p "${EXTENSION_DIR}"
    mkdir -p "${LIB_DIR}"

    cp "${SQLGEN_CONTROL_PATH}" "${EXTENSION_DIR}"
    cp "${SQLGEN_SQL_PATH}" "${EXTENSION_DIR}"
    cp "${SQLGEN_SO_PATH}" "${LIB_DIR}"
done

rm -rf pg-sqlgen_${BUILD_VERSION}_amd64.deb
rm -rf pg_sqlgen-${BUILD_VERSION}-1.x86_64.rpm

PACKAGE_TARGET=./target/packages
mkdir -p ${PACKAGE_TARGET}

for t in "${PKG_PLATFORMS[@]}"; do
    out="$(
        fpm \
            -s dir \
            -t "$t" \
            -n pg_sqlgen \
            -v "${BUILD_VERSION}" \
            -C target/package \
            --license MIT \
            --description "Text-to-SQL extension for Postgres" \
            --url "https://github.com/iakinsey/pg_sqlgen" \
            --maintainer "Ian Kinsey <ian@aikbix.com>" \
            . 2>&1
    )"

    echo $out

    pkg_path=$(sed -n 's/.*path: "\(.*\)".*/\1/p' <<< "$out")
    mv ${pkg_path} ${PACKAGE_TARGET}
done