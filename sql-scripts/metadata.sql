--------------------------------------------------------------------------------
-- Initialize metadata
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.initialize_metadata(
    engine TEXT,
    schema_name TEXT,
    vector_size INT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM sqlgen_internal.create_metadata_table(engine, schema_name, vector_size);
  PERFORM sqlgen_internal.install_schema_triggers(engine, schema_name);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.initialize_metadata(
    TEXT, TEXT, INT
) FROM public;

--------------------------------------------------------------------------------
-- Remove metadata
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_metadata(
    engine TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM sqlgen_internal.remove_schema_triggers(engine, schema_name);
  PERFORM sqlgen_internal.remove_metadata_table(engine, schema_name);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_metadata FROM public;

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
DECLARE
  vector_column_type TEXT;
BEGIN
  SELECT
    sqlgen_internal.get_vector_column_type(vector_size)
  INTO vector_column_type;

  EXECUTE format($fmt$
    CREATE TABLE sqlgen_internal.db_metadata_%I (
      schema_name     TEXT NOT NULL,
      table_name      TEXT NOT NULL,
      column_name     TEXT NOT NULL,
      ddl             TEXT NOT NULL,
      comment         TEXT,
      ddl_vector      %s NOT NULL,
      comment_vector  %s
    );
  $fmt$, engine, vector_column_type, vector_column_type);

  EXECUTE format($fmt$
    REVOKE ALL ON TABLE sqlgen_internal.db_metadata_%I FROM PUBLIC;
  $fmt$, engine);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_metadata_table FROM public;

--------------------------------------------------------------------------------
-- Get similar ddls
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_similar_ddls(
    engine TEXT,
    user_query FLOAT4 [],
    similarity_limit INT
)
RETURNS SETOF TEXT
LANGUAGE plpgsql
AS $$
DECLARE
  use_vec_search BOOLEAN;
BEGIN
  use_vec_search := sqlgen_internal.use_vector_search();

  IF use_vec_search THEN
    RETURN QUERY EXECUTE format(
      'SELECT ddl
         FROM %I.%I
         ORDER BY ddl_vector <-> $1::VECTOR
         LIMIT $2',
      'sqlgen_internal',
      'db_metadata_' || engine
    )
    USING user_query, similarity_limit;
  ELSE
    RETURN QUERY EXECUTE format(
      'SELECT ddl
         FROM %I.%I
         ORDER BY sqlgen.cosine_distance($1::float4[], ddl_vector)
         LIMIT $2',
      'sqlgen_internal',
      'db_metadata_' || engine
    )
    USING user_query, similarity_limit;
  END IF;
END
$$;


REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_similar_ddls FROM public;

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

REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_ddls FROM public;

--------------------------------------------------------------------------------
-- Install schema triggers. These are used to update metadata tables when 
-- entities in a schema changes.
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.install_schema_triggers(
    engine TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
    -- Drop existing event triggers if present
    DROP EVENT TRIGGER IF EXISTS trigger_create_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_alter_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_drop_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_comment_%1$I;

    -- Create table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_create_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      r RECORD;
      v_tbl text;
    BEGIN
      FOR r IN
        SELECT schema_name,
               substring(object_identity FROM '[^.]+$') AS table_name
        FROM pg_event_trigger_ddl_commands()
        WHERE object_type IN ('table','partitioned table')
      LOOP
        v_tbl := r.table_name;

        IF r.schema_name = %2$L THEN
          PERFORM sqlgen_internal.add_table(%1$L, %2$L, v_tbl);
        END IF;
      END LOOP;
    END; $fn$;

    -- Alter table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_alter_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      r RECORD;
      v_tbl text;
    BEGIN
      FOR r IN
        SELECT schema_name,
               substring(object_identity FROM '[^.]+$') AS table_name
        FROM pg_event_trigger_ddl_commands()
        WHERE object_type IN ('table','partitioned table')
      LOOP
        v_tbl := r.table_name;

        IF r.schema_name = %2$L THEN
          PERFORM sqlgen_internal.update_table(%1$L, %2$L, v_tbl);
        END IF;
      END LOOP;
    END; $fn$;

    -- Drop table function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_drop_table_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    DECLARE
      r RECORD;
      v_tbl text;
    BEGIN
      FOR r IN
        SELECT schema_name, object_name AS table_name
        FROM pg_event_trigger_dropped_objects()
        WHERE object_type IN ('table','partitioned table')
      LOOP
        v_tbl := r.table_name;

        IF r.schema_name = %2$L THEN
          PERFORM sqlgen_internal.remove_table(%1$L, %2$L, v_tbl);
        END IF;
      END LOOP;
    END; $fn$;

    -- Comment function
    CREATE OR REPLACE FUNCTION sqlgen_internal.on_comment_%1$I()
    RETURNS event_trigger
    LANGUAGE plpgsql AS $fn$
    BEGIN
      PERFORM sqlgen_internal.update_comment(%1$L, %2$L);
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

     CREATE EVENT TRIGGER trigger_comment_%1$I
      ON ddl_command_end
      WHEN TAG IN ('COMMENT')
      EXECUTE FUNCTION sqlgen_internal.on_comment_%1$I();

    -- Drop table trigger
    CREATE EVENT TRIGGER trigger_drop_table_%1$I
      ON sql_drop
      EXECUTE FUNCTION sqlgen_internal.on_drop_table_%1$I();
  $fmt$,
    engine,       -- %1$*
    schema_name   -- %2$*
  );
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.install_schema_triggers FROM public;

--------------------------------------------------------------------------------
-- Remove metadata table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_metadata_table(
    engine TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  -- Drop the metadata table for this engine
  EXECUTE format($fmt$
    DROP TABLE IF EXISTS sqlgen_internal.db_metadata_%I
  $fmt$, engine);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_metadata_table FROM public;

--------------------------------------------------------------------------------
-- Crawl schema. Generates data for entities in a given schema to help
-- populate metadata tables.
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.crawl_schema(schema_name TEXT)
RETURNS TABLE (
    table_name TEXT,
    column_name TEXT,
    ddl TEXT,
    comment TEXT
)
LANGUAGE sql
AS $$
SELECT
  c.relname AS table_name,
  a.attname AS column_name,
  (
    format('%I.%I.%I %I %s',
      n.nspname,
      c.relname,
      a.attname,
      a.attname,
      format_type(a.atttypid, a.atttypmod)
    )
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

REVOKE EXECUTE ON FUNCTION sqlgen_internal.crawl_schema FROM public;

--------------------------------------------------------------------------------
-- Remove table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_table(
    engine TEXT,
    schema_name TEXT,
    table_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
  DELETE FROM sqlgen_internal.db_metadata_%I
  WHERE schema_name = %L
    AND table_name = %L
  $fmt$, engine, schema_name, table_name);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_table FROM public;

--------------------------------------------------------------------------------
-- Update table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.update_table(
    engine TEXT,
    schema_name TEXT,
    table_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  PERFORM sqlgen_internal.remove_table(engine, schema_name, table_name);
  PERFORM sqlgen_internal.add_table(engine, schema_name, table_name);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.update_table FROM public;

--------------------------------------------------------------------------------
-- Remove schema triggers
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_schema_triggers(
    engine TEXT,
    schema_name TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
    -- Drop event triggers
    DROP EVENT TRIGGER IF EXISTS trigger_create_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_alter_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_drop_table_%1$I;
    DROP EVENT TRIGGER IF EXISTS trigger_comment_%1$I;

    -- Drop associated event trigger functions
    DROP FUNCTION IF EXISTS sqlgen_internal.on_create_table_%1$I();
    DROP FUNCTION IF EXISTS sqlgen_internal.on_alter_table_%1$I();
    DROP FUNCTION IF EXISTS sqlgen_internal.on_drop_table_%1$I();
    DROP FUNCTION IF EXISTS sqlgen_internal.on_comment_%1$I();
  $fmt$, engine);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_schema_triggers FROM public;


--------------------------------------------------------------------------------
-- Update comments
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.update_comment(
    engine TEXT, expected_schema TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
  r RECORD;
  v_comment TEXT;
  v_schema TEXT;
  v_table TEXT;
  v_column TEXT;
BEGIN
  FOR r IN
    SELECT classid, objid, objsubid, object_identity, schema_name
    FROM pg_event_trigger_ddl_commands()
    WHERE command_tag = 'COMMENT'
  LOOP
    SELECT d.description
      INTO v_comment
      FROM pg_description d
     WHERE d.classoid = r.classid
       AND d.objoid  = r.objid
       AND d.objsubid = COALESCE(r.objsubid, 0);

    IF v_comment IS NULL THEN
      SELECT sd.description
        INTO v_comment
        FROM pg_shdescription sd
       WHERE sd.classoid = r.classid
         AND sd.objoid  = r.objid;
    END IF;

    v_schema := COALESCE(r.schema_name, 'public');
    v_table := (string_to_array(r.object_identity, '.'))[array_length(string_to_array(r.object_identity, '.'), 1) - 1];
    v_column := (string_to_array(r.object_identity, '.'))[array_length(string_to_array(r.object_identity, '.'), 1)];

    IF r.schema_name = expected_schema THEN
      PERFORM sqlgen_internal.do_update_comment(
        engine,
        v_schema,
        v_table,
        v_column,
        v_comment
      );
    END IF;
  END LOOP;
END;
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.update_comment FROM public;

--------------------------------------------------------------------------------
-- Update comment event
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.do_update_comment(
    engine TEXT,
    schema_name TEXT,
    table_name TEXT,
    column_name TEXT,
    comment TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
  comment_vector TEXT;
  vector_column_type TEXT;
BEGIN
  comment_vector := sqlgen_internal.encode_text(engine, comment);
  vector_column_type := sqlgen_internal.get_vector_column_type(vector_size);

  EXECUTE format($fmt$
    UPDATE sqlgen_internal.db_metadata_%I
    SET
      comment = %L,
      comment_vector = %L::%s
    WHERE
      table_name = %L
      AND column_name = %L
  $fmt$, engine, comment, comment_vector, vector_column_type, table_name, column_name);
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.do_update_comment FROM public;
