#!/usr/bin/env bash
set -euo pipefail

PG_MAJOR=${PG_MAJOR:-16}

PG_BIN="/usr/pgsql-${PG_MAJOR}/bin"
PG_DATA="/var/lib/pgsql/${PG_MAJOR}/data"

# Initialize cluster if needed
if [ ! -f "${PG_DATA}/PG_VERSION" ]; then
  mkdir -p "${PG_DATA}"
  chown postgres:postgres "${PG_DATA}"
  runuser -u postgres -- "${PG_BIN}/initdb" -D "${PG_DATA}"
fi

# Start PostgreSQL
runuser -u postgres -- "${PG_BIN}/pg_ctl" -D "${PG_DATA}" -w start

# Wait until ready
until runuser -u postgres -- "${PG_BIN}/pg_isready" -q; do
  sleep 1
done

# Run SQL as postgres
runuser -u postgres -- "${PG_BIN}/psql" -v ON_ERROR_STOP=1 <<'SQL'
CREATE EXTENSION sqlgen CASCADE;
SELECT sqlgen.add_model(
  'test',
  sqlgen.stub_config('hello world', ARRAY[0.0, 0.5, 1.0]::REAL[])
);
SQL

# Stop PostgreSQL
runuser -u postgres -- "${PG_BIN}/pg_ctl" -D "${PG_DATA}" -m fast -w stop
