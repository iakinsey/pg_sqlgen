CREATE OR REPLACE FUNCTION sqlgen_internal.crawl_schema(schema_name TEXT)
RETURNS TABLE (
  schema_name  TEXT,
  table_name   TEXT,
  column_name  TEXT,
  ddl          TEXT,
  comment      TEXT
)
SELECT
  n.nspname AS Schema_Name,
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

REVOKE EXECUTE ON FUNCTION sqlgen_internal.crawl_schema(text) FROM PUBLIC;