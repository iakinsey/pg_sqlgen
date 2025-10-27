--------------------------------------------------------------------------------
-- Text to SQL engine
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.engine (
    engine_name TEXT PRIMARY KEY,
    schema_name TEXT NOT NULL,
    encoder_model TEXT NOT NULL,
    instruct_model TEXT NOT NULL,
    system_prompt_template TEXT,
    user_prompt_template TEXT,
    relevant_ddls_template TEXT,
    similar_queries_template TEXT,
    filter_ddls_template TEXT,
    table_filter_type TEXT,
    CONSTRAINT fk_encoder_model
        FOREIGN KEY (encoder_model)
        REFERENCES sqlgen_internal.model_profile(model_name)
        ON DELETE CASCADE
        ON UPDATE CASCADE,
    CONSTRAINT fk_instruct_model
        FOREIGN KEY (instruct_model)
        REFERENCES sqlgen_internal.model_profile(model_name)
        ON DELETE CASCADE
        ON UPDATE CASCADE
);

REVOKE ALL ON TABLE sqlgen_internal.engine FROM public;

--------------------------------------------------------------------------------
-- Engines view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.engines AS
    SELECT * FROM sqlgen_internal.engine;

--------------------------------------------------------------------------------
-- Create engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_engine(
    engine_name TEXT,
    schema_name TEXT,
    encoder_model TEXT,
    instruct_model TEXT,
    table_filter_type TEXT DEFAULT 'smart',
    system_prompt_template TEXT DEFAULT NULL,
    user_prompt_template TEXT DEFAULT NULL,
    relevant_ddls_template TEXT DEFAULT NULL,
    similar_queries_template TEXT DEFAULT NULL,
    filter_ddls_template TEXT DEFAULT NULL
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
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_ddls_template,
        table_filter_type
    ) VALUES (
        engine_name,
        schema_name,
        encoder_model,
        instruct_model,
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_ddls_template,
        table_filter_type
    );
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_engine(
    TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TEXT, TEXT
) FROM public;

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
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_engine(TEXT) FROM public;

--------------------------------------------------------------------------------
-- Get engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_engine(n TEXT)
RETURNS sqlgen_internal.engine
LANGUAGE plpgsql
AS $$
DECLARE
    r sqlgen_internal.engine%ROWTYPE;
BEGIN
    SELECT *
    INTO r
    FROM sqlgen_internal.engine e
    WHERE e.engine_name = n;

    RETURN r;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_engine(TEXT) FROM public;

--------------------------------------------------------------------------------
-- List engines
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.list_engines()
RETURNS SETOF sqlgen_internal.engine
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM sqlgen_internal.engine;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.list_engines() FROM public;