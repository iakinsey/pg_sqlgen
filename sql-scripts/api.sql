--------------------------------------------------------------------------------
/*
    Generates an SQL query from text and execute it

    Example call:

    BEGIN;
        SELECT sqlgen.query('Get highest paying customer');
        FETCH ALL FROM sqlgen_query;
    COMMIT;
*/
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

    c := 'sqlgen_query';
    OPEN c FOR EXECUTE dyn_sql;

    RETURN c;
END;
$$;