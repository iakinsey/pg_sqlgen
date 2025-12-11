--------------------------------------------------------------------------------
/*
    Generates an SQL query from text and execute it.

    Example call:

    BEGIN;
        SELECT sqlgen.query('Get highest paying customer');
        FETCH ALL FROM sqlgen_query;
    COMMIT;
*/
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.query(
    user_query TEXT,
    engine TEXT DEFAULT NULL
)
RETURNS REFCURSOR
LANGUAGE plpgsql
AS $$
DECLARE
    dyn_sql TEXT;
    c       refcursor;
BEGIN
    SELECT sqlgen.generate(user_query, engine) INTO dyn_sql;

    c := 'sqlgen_query';
    OPEN c FOR EXECUTE dyn_sql;

    RETURN c;
END;
$$;


--------------------------------------------------------------------------------
-- Create new text-to-sql engine.
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.create_engine(
    name TEXT,
    instruct_model TEXT,
    encoder_model TEXT,
    schema_name TEXT DEFAULT NULL,
    table_filter_type TEXT DEFAULT 'smart',
    column_filter_limit INTEGER DEFAULT NULL,
    error_correction_rounds INTEGER DEFAULT NULL,
    system_prompt_template TEXT DEFAULT NULL,
    user_prompt_template TEXT DEFAULT NULL,
    relevant_ddls_template TEXT DEFAULT NULL,
    similar_queries_template TEXT DEFAULT NULL,
    filter_ddls_template TEXT DEFAULT NULL,
    syntax_correction_template TEXT DEFAULT NULL,
    explain_query_template TEXT DEFAULT NULL
)
RETURNS VOID
LANGUAGE sql
AS $$
    SELECT sqlgen_internal.create_engine_external(
        name,
        instruct_model,
        encoder_model,
        schema_name,
        table_filter_type,
        error_correction_rounds,
        column_filter_limit,
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_ddls_template,
        syntax_correction_template,
        explain_query_template
    );
$$;

--------------------------------------------------------------------------------
-- Change vector backend
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.set_vector_backend(
    instance TEXT DEFAULT NULL
)
RETURNS TEXT
LANGUAGE plpgsql
AS $$
DECLARE
    result TEXT;
BEGIN
    SELECT sqlgen_internal.set_vector_backend_external(instance) INTO result;
    RETURN result;
END
$$;
