CREATE OR REPLACE FUNCTION sqlgen_internal.create_metadata_table(unique_name TEXT, vector_size INT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE FORMAT($fmt$
    CREATE TABLE sqlgen_internal.db_metadata_%I (
      schema_name     TEXT NOT NULL,
      table_name      TEXT NOT NULL,
      column_name     TEXT NOT NULL,
      ddl             TEXT NOT NULL,
      comment         TEXT,
      ddl_vector      VECTOR(%s) NOT NULL,
      comment_vector  VECTOR(%s)
    );
  $fmt$, unique_name, vector_size, vector_size);

  EXECUTE FORMAT($fmt$
    REVOKE ALL ON TABLE sqlgen_internal.db_metadata_%I FROM PUBLIC;
  $fmt$, unique_name);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_metadata_table(TEXT, INT) FROM public;


CREATE OR REPLACE FUNCTION sqlgen_internal.install_schema_triggers(schema_name TEXT, unique_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  -- TODO
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.install_schema_triggers(TEXT, TEXT) FROM public;


CREATE OR REPLACE FUNCTION sqlgen_internal.remove_schema_triggers(schema_name TEXT, unique_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  -- TODO
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_schema_triggers(TEXT, TEXT) FROM public;



CREATE OR REPLACE FUNCTION sqlgen_internal.remove_metadata_table(unique_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  DO $triggers$
    DECLARE
      s RECORD;
    BEGIN
      FOR s IN
        SELECT schema_name
        FROM information_schema.schemata
      LOOP
        BEGIN
          PERFORM sqlgen_internal.remove_schema_triggers(s.schema_name);
        EXCEPTION WHEN OTHERS THEN
          -- suppress errors
          NULL;
        END;
      END LOOP;
    END;
  $triggers$;

  EXECUTE FORMAT($fmt$
    DROP TABLE sqlgen_internal.db_metadata_%I;
  $fmt$, unique_name);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_metadata_table(TEXT) FROM public;


CREATE OR REPLACE FUNCTION sqlgen_internal.crawl_schema(schema_name TEXT)
RETURNS TABLE (
  table_name   TEXT,
  column_name  TEXT,
  ddl          TEXT,
  comment      TEXT
)
SELECT
  c.relname AS Table_Name,
  a.attname AS Column_Name,
  (
    format('%I %s', a.attname, format_type(a.atttypid, a.atttypmod))

    || CASE WHEN a.attidentity IN ('a','d') AND coalesce(a.attgenerated,'') = '' THEN
         ' GENERATED ' || CASE a.attidentity WHEN 'a' THEN 'ALWAYS' ELSE 'BY DEFAULT' END || ' AS IDENTITY'
       ELSE '' END

    || CASE WHEN a.attgenerated = 's' THEN
         ' GENERATED ALWAYS AS (' || pg_get_expr(ad.adbin, ad.adrelid) || ') STORED'
       ELSE '' END

    || CASE WHEN ad.adbin IS NOT NULL AND coalesce(a.attgenerated,'') = '' THEN
         ' DEFAULT ' || pg_get_expr(ad.adbin, ad.adrelid)
       ELSE '' END

    || CASE WHEN a.attnotnull THEN ' NOT NULL' ELSE '' END
  )                                         AS DDL,
  pg_catalog.col_description(c.oid, a.attnum) AS Comment
FROM pg_attribute      a
JOIN pg_class          c  ON c.oid = a.attrelid
JOIN pg_namespace      n  ON n.oid = c.relnamespace
LEFT JOIN pg_attrdef   ad ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
WHERE n.nspname = schema_name
  AND c.relkind IN ('r','p','v','m')
  AND a.attnum > 0
  AND NOT a.attisdropped
ORDER BY n.nspname, c.relname, a.attnum;
LANGUAGE SQL
AS $$
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.crawl_schema(text) FROM public;