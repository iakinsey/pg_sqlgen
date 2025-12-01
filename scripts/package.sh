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

    # Debian root structure
    EXTENSION_DEBIAN_DIR=./target/package-deb/usr/share/postgresql/${v}/extension/
    LIB_DEBIAN_DIR=./target/package-deb/usr/lib/postgresql/${v}/lib/

    rm -rf "${EXTENSION_DEBIAN_DIR}"
    rm -rf "${LIB_DEBIAN_DIR}"

    mkdir -p "${EXTENSION_DEBIAN_DIR}"
    mkdir -p "${LIB_DEBIAN_DIR}"

    cp "${SQLGEN_CONTROL_PATH}" "${EXTENSION_DEBIAN_DIR}"
    cp "${SQLGEN_SQL_PATH}" "${EXTENSION_DEBIAN_DIR}"
    cp "${SQLGEN_SO_PATH}" "${LIB_DEBIAN_DIR}"

    # Redhat root structure
    REDHAT_CONTROL_DIR=./target/package-rpm/usr/pgsql-${v}/share/extension
    REDHAT_SQL_DIR=./target/package-rpm/usr/pgsql-${v}/share/extension
    REDHAT_SO_DIR=./target/package-rpm/usr/pgsql-${v}/lib

    rm -rf "${REDHAT_CONTROL_DIR}"
    rm -rf "${REDHAT_SQL_DIR}"
    rm -rf "${REDHAT_SO_DIR}"

    mkdir -p "${REDHAT_CONTROL_DIR}"
    mkdir -p "${REDHAT_SQL_DIR}"
    mkdir -p "${REDHAT_SO_DIR}"

    cp "${SQLGEN_CONTROL_PATH}" "${REDHAT_CONTROL_DIR}"
    cp "${SQLGEN_SQL_PATH}" "${REDHAT_SQL_DIR}"
    cp "${SQLGEN_SO_PATH}" "${REDHAT_SO_DIR}"
done

rm -rf pg-sqlgen_${BUILD_VERSION}_amd64.deb
rm -rf pg_sqlgen-${BUILD_VERSION}-1.x86_64.rpm

PACKAGE_TARGET=./target/packages
mkdir -p ${PACKAGE_TARGET}


if [[ " ${PKG_PLATFORMS[*]} " == *" deb "* ]]; then
    out="$(
        fpm \
            -s dir \
            -t deb \
            -n pg_sqlgen \
            -v "${BUILD_VERSION}" \
            -C target/package-deb \
            --license MIT \
            --description "Text-to-SQL extension for Postgres" \
            --url "https://github.com/iakinsey/pg_sqlgen" \
            --maintainer "Ian Kinsey <ian@aikbix.com>" \
            . 2>&1
    )"

    echo $out

    pkg_path=$(sed -n 's/.*path: "\(.*\)".*/\1/p' <<< "$out")
    mv ${pkg_path} ${PACKAGE_TARGET}
fi

if [[ " ${PKG_PLATFORMS[*]} " == *" rpm "* ]]; then
    out="$(
        fpm \
            -s dir \
            -t rpm \
            -n pg_sqlgen \
            -v "${BUILD_VERSION}" \
            -C target/package-rpm \
            --license MIT \
            --description "Text-to-SQL extension for Postgres" \
            --url "https://github.com/iakinsey/pg_sqlgen" \
            --maintainer "Ian Kinsey <ian@aikbix.com>" \
            . 2>&1
    )"

    echo $out

    pkg_path=$(sed -n 's/.*path: "\(.*\)".*/\1/p' <<< "$out")
    mv ${pkg_path} ${PACKAGE_TARGET}
fi
