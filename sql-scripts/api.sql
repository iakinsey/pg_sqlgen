--------------------------------------------------------------------------------
-- Generates an SQL query from text and execute it
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.query(
    user_query TEXT,
    engine     TEXT DEFAULT NULL
)
RETURNS refcursor
LANGUAGE plpgsql
AS $$
DECLARE
    dyn_sql TEXT;
    c       refcursor;
BEGIN
    SELECT generate(user_query, engine) INTO dyn_sql;

    c := 'sqlgen_query_cursor';
    OPEN c FOR EXECUTE dyn_sql;

    RETURN c;
END;
$$;