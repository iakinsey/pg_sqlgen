#!/usr/bin/env bash
set -euo pipefail

PG_MAJOR=${PG_MAJOR:-16}

if [ ! -d "/var/lib/postgresql/${PG_MAJOR}/main" ]; then
  /usr/lib/postgresql/${PG_MAJOR}/bin/pg_createcluster "${PG_MAJOR}" main
fi

service postgresql start

until pg_isready -U postgres >/dev/null 2>&1; do
  sleep 1
done

# Run SQL as the postgres OS user to satisfy peer auth
runuser -u postgres -- psql -v ON_ERROR_STOP=1 <<'SQL'
CREATE EXTENSION sqlgen CASCADE;
SELECT sqlgen.add_model(
  'test',
  sqlgen.stub_config('hello world', ARRAY[0.0, 0.5, 1.0]::REAL[])
);
SQL

service postgresql stop
