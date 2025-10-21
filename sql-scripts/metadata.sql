--------------------------------------------------------------------------------
-- Initialize metadata
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.initialize_metadata(
    engine TEXT,
    model_name TEXT,
    schema_name TEXT,
    vector_size INT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM sqlgen_internal.create_metadata_table(engine, schema_name, vector_size);
  PERFORM sqlgen_internal.install_schema_triggers(engine, model_name, schema_name);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.initialize_metadata(TEXT, TEXT, TEXT, INT) FROM public;

--------------------------------------------------------------------------------
-- Remove metadata
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.remove_metadata(
    engine TEXT,
    model_name TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM sqlgen_internal.remove_schema_triggers(engine, model_name, schema_name);
  PERFORM sqlgen_internal.remove_metadata_table(engine, model_name, schema_name);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_metadata(TEXT, TEXT, TEXT) FROM public;

--------------------------------------------------------------------------------
-- Create metadata table
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.create_metadata_table(
    engine TEXT,
    schema_name TEXT,
    vector_size INT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
    CREATE TABLE sqlgen_internal.db_metadata_%I (
      schema_name     TEXT NOT NULL,
      table_name      TEXT NOT NULL,
      column_name     TEXT NOT NULL,
      ddl             TEXT NOT NULL,
      comment         TEXT,
      ddl_vector      VECTOR(%s) NOT NULL,
      comment_vector  VECTOR(%s)
    );
  $fmt$, engine, vector_size, vector_size);

  EXECUTE format($fmt$
    REVOKE ALL ON TABLE sqlgen_internal.db_metadata_%I FROM PUBLIC;
  $fmt$, engine);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_metadata_table(TEXT, TEXT, INT) FROM public;

--------------------------------------------------------------------------------
-- Get similar ddls
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.get_similar_ddls(
  engine TEXT,
  user_query VECTOR,
  similarity_limit INT
)
RETURNS SETOF TEXT
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN QUERY EXECUTE format(
    'SELECT ddl
       FROM %I.%I
       ORDER BY ddl_vector <-> $1
       LIMIT $2',
    'sqlgen_internal',
    'db_metadata_' || engine
  )
  USING user_query, similarity_limit;
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_similar_ddls(TEXT, VECTOR, INT) FROM public;

--------------------------------------------------------------------------------
-- Get ddls
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.get_ddls(
  engine TEXT
)
RETURNS SETOF TEXT
LANGUAGE plpgsql
AS $$
BEGIN
  RETURN QUERY EXECUTE format($fmt$
    SELECT ddl FROM sqlgen_internal.db_metadata_%I
  $fmt$, engine);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_ddls(TEXT) FROM public;

--------------------------------------------------------------------------------
-- Install schema triggers
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.install_schema_triggers(
    engine TEXT,
    model_name TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  RAISE NOTICE '!!!!!!111 :D';
  EXECUTE format($fmt$
    -- Drop existing event triggers if present
    DROP EVENT TRIGGER IF EXISTS trigger_create_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_alter_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_drop_table_%1$I;

    -- Create table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_create_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    BEGIN
      RAISE NOTICE 'CREATE %%',
        (SELECT json_agg(json_build_object('schema', %2$L, 'object', object_identity))
         FROM pg_event_trigger_ddl_commands()
         WHERE object_type IN ('table','partitioned table'));
    END; $fn$;

    -- Alter table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_alter_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    BEGIN
      RAISE NOTICE 'ALTER %%',
        (SELECT json_agg(json_build_object('schema', %2$L, 'object', object_identity))
         FROM pg_event_trigger_ddl_commands()
         WHERE object_type IN ('table','partitioned table'));
    END; $fn$;

    -- Drop table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_drop_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    BEGIN
      RAISE NOTICE 'DROP %%',
        (SELECT json_agg(json_build_object('schema', %2$L, 'object', object_identity))
         FROM pg_event_trigger_dropped_objects()
         WHERE object_type IN ('table','partitioned table'));
    END; $fn$;

    -- Create table trigger
    CREATE EVENT TRIGGER trigger_create_table_%1$I
      ON ddl_command_end
      WHEN TAG IN ('CREATE TABLE')
      EXECUTE FUNCTION sqlgen_internal.on_create_table_%1$I();

    -- Alter table trigger
    CREATE EVENT TRIGGER trigger_alter_table_%1$I
      ON ddl_command_end
      WHEN TAG IN ('ALTER TABLE')
      EXECUTE FUNCTION sqlgen_internal.on_alter_table_%1$I();

    -- Drop table trigger
    CREATE EVENT TRIGGER trigger_drop_table_%1$I
      ON sql_drop
      EXECUTE FUNCTION sqlgen_internal.on_drop_table_%1$I();
  $fmt$,
    engine,         -- %1$I
    schema_name     -- %2$L
  );
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.install_schema_triggers(TEXT, TEXT, TEXT) FROM public;

    RETURNS VOID



   
   
   


LANGUAGE plpgsql
AS $$
BEGIN
  -- TODO: drop specific triggers here
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_schema_triggers(TEXT, TEXT, TEXT) FROM public;

--------------------------------------------------------------------------------
-- Remove metadata table
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.remove_metadata_table(
    engine TEXT,
    model_name TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  -- Remove triggers for the specified schema/model
  PERFORM sqlgen_internal.remove_schema_triggers(engine, model_name, schema_name);

  -- Drop the metadata table for this engine
  EXECUTE format($fmt$
    DROP TABLE IF EXISTS sqlgen_internal.db_metadata_%I
  $fmt$, engine);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_metadata_table(TEXT, TEXT, TEXT) FROM public;

--------------------------------------------------------------------------------
-- Crawl schema
--------------------------------------------------------------------------------
CREATE OR REPLACE FUNCTION sqlgen_internal.crawl_schema(schema_name TEXT)
RETURNS TABLE (
  table_name   TEXT,
  column_name  TEXT,
  ddl          TEXT,
  comment      TEXT
)
LANGUAGE sql
AS $$
SELECT
  c.relname AS table_name,
  a.attname AS column_name,
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
  ) AS ddl,
  pg_catalog.col_description(c.oid, a.attnum) AS comment
FROM pg_attribute      a
JOIN pg_class          c  ON c.oid = a.attrelid
JOIN pg_namespace      n  ON n.oid = c.relnamespace
LEFT JOIN pg_attrdef   ad ON ad.adrelid = a.attrelid AND ad.adnum = a.attnum
WHERE n.nspname = $1
  AND c.relkind IN ('r','p','v','m')
  AND a.attnum > 0
  AND NOT a.attisdropped
ORDER BY n.nspname, c.relname, a.attnum;
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.crawl_schema(TEXT) FROM public;


--------------------------------------------------------------------------------
-- Add table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.add_table_metadata(
    engine TEXT,
    table_name TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN


END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_schema_triggers(TEXT, TEXT, TEXT) FROM public;


--------------------------------------------------------------------------------
-- Remove table
--------------------------------------------------------------------------------

--------------------------------------------------------------------------------
-- Update table
--------------------------------------------------------------------------------

