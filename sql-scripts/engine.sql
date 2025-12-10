--------------------------------------------------------------------------------
-- Text to SQL engine
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.engine (
    engine_name TEXT PRIMARY KEY,
    schema_name TEXT NOT NULL,
    encoder_model TEXT NOT NULL,
    instruct_model TEXT NOT NULL,
    column_filter_limit INT NOT NULL,
    error_correction_rounds INT NOT NULL,
    system_prompt_template TEXT,
    user_prompt_template TEXT,
    relevant_ddls_template TEXT,
    similar_queries_template TEXT,
    filter_ddls_template TEXT,
    syntax_correction_template TEXT,
    explain_query_template TEXT,
    table_filter_type TEXT,
    CONSTRAINT fk_encoder_model
    FOREIGN KEY (encoder_model)
    REFERENCES sqlgen_internal.model_profile (model_name)
    ON DELETE CASCADE
    ON UPDATE CASCADE,
    CONSTRAINT fk_instruct_model
    FOREIGN KEY (instruct_model)
    REFERENCES sqlgen_internal.model_profile (model_name)
    ON DELETE CASCADE
    ON UPDATE CASCADE
);

REVOKE ALL ON TABLE sqlgen_internal.engine FROM public;

--------------------------------------------------------------------------------
-- Engines view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.engines AS
SELECT * FROM sqlgen_internal.engine; -- noqa: AM04

--------------------------------------------------------------------------------
-- Create engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_engine(
    engine_name TEXT,
    schema_name TEXT,
    encoder_model TEXT,
    instruct_model TEXT,
    table_filter_type TEXT DEFAULT 'smart',
    column_filter_limit INT DEFAULT NULL,
    error_correction_rounds INT DEFAULT NULL,
    system_prompt_template TEXT DEFAULT NULL,
    user_prompt_template TEXT DEFAULT NULL,
    relevant_ddls_template TEXT DEFAULT NULL,
    similar_queries_template TEXT DEFAULT NULL,
    filter_ddls_template TEXT DEFAULT NULL,
    syntax_correction_template TEXT DEFAULT NULL,
    explain_query_template TEXT DEFAULT NULL
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO sqlgen_internal.engine (
        engine_name,
        schema_name,
        encoder_model,
        instruct_model,
        table_filter_type,
        column_filter_limit,
        error_correction_rounds,
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_ddls_template,
        syntax_correction_template,
        explain_query_template
    ) VALUES (
        engine_name,
        schema_name,
        encoder_model,
        instruct_model,
        table_filter_type,
        column_filter_limit,
        error_correction_rounds,
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_ddls_template,
        syntax_correction_template,
        explain_query_template
    );
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_engine FROM public;

--------------------------------------------------------------------------------
-- Remove engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_engine(n TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM sqlgen_internal.engine e
    WHERE e.engine_name = n;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_engine FROM public;

--------------------------------------------------------------------------------
-- Get engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_engine(n TEXT)
RETURNS SETOF sqlgen_internal.ENGINE
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT e.*
    FROM sqlgen_internal.engine e
    WHERE e.engine_name = n;
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_engine FROM public;


--------------------------------------------------------------------------------
-- List engines
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.list_engines()
RETURNS SETOF sqlgen_internal.ENGINE
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM sqlgen_internal.engine;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.list_engines FROM public;


--------------------------------------------------------------------------------
-- Engine uses model
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_engines_used_by_model(
    model_name TEXT
)
RETURNS TEXT []
LANGUAGE plpgsql
AS $$
DECLARE
    engines TEXT[];
BEGIN
    SELECT ARRAY(
        SELECT engine_name
        FROM sqlgen_internal.engine
        WHERE encoder_model = model_name
           OR instruct_model = model_name
    )
    INTO engines;

    RETURN engines;
END
$$;
