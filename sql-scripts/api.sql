--------------------------------------------------------------------------------
-- Generates an SQL query from text and execute it
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.query(user_query TEXT, engine TEXT DEFAULT NULL)
RETURNS SETOF RECORD
LANGUAGE plpgsql
AS $$
DECLARE
    dyn_sql TEXT;
BEGIN
    SELECT generate(user_query, engine) INTO dyn_sql;
    RETURN QUERY EXECUTE dyn_sql;
END;
$$;