--------------------------------------------------------------------------------
-- Text to SQL engine
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.engine (
    engine_name TEXT NOT NULL PRIMARY KEY,
    schema_name TEXT NOT NULL,
    encoder_model TEXT NOT NULL,
    instruct_model TEXT NOT NULL,
    system_prompt_template TEXT,
    user_prompt_template TEXT,
    relevant_ddls_template TEXT,
    similar_queries_template TEXT
    filter_prompt_template TEXT,
    table_filter_type TEXT
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
REVOKE ALL ON TABLE sqlgen_internal.engine FROM PUBLIC;

--------------------------------------------------------------------------------
-- Engines view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.engines {
    SELECT * FROM sqlgen_internal.model_profile
}

--------------------------------------------------------------------------------
-- Create engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_engine(
    engine_name TEXT,
    schema_name TEXT,
    encoder_model TEXT,
    instruct_model TEXT,
    system_prompt_template TEXT DEFAULT NULL,
    user_prompt_template TEXT DEFAULT NULL,
    relevant_ddls_template TEXT DEFAULT NULL,
    similar_queries_template TEXT DEFAULT NULL,
    filter_prompt_template TEXT DEFAULT NULL,
    table_filter_type TEXT DEFAULT 'smart'
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
        filter_prompt_template,
        filter_type
    ) VALUES (
        engine_name,
        schema_name,
        encoder_model,
        instruct_model,
        system_prompt_template,
        user_prompt_template,
        relevant_ddls_template,
        similar_queries_template,
        filter_prompt_template,
        filter_type
    );
EXCEPTION
    WHEN unique_violation THEN
        RAISE EXCEPTION 'Engine "%" already exists', engine_name
            USING ERRCODE = 'unique_violation';
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_engine(TEXT, TEXT, TEXT, TEXT) FROM public;

--------------------------------------------------------------------------------
-- Remove engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_engine(model_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    DELETE FROM sqlgen_internal.engine
    WHERE engine_name = remove_engine.engine_name;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Engine "%" does not exist', engine_name
            USING ERRCODE = 'no_data_found';
    END IF;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.remove_engine(TEXT) FROM public;

--------------------------------------------------------------------------------
-- Get engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_engine(engine TEXT)
RETURNS sqlgen_internal.engine
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM sqlgen_internal.engine e
    WHERE e.engine_name = get_engine.engine_name;

    IF NOT FOUND THEN
        RAISE EXCEPTION 'Engine "%" does not exist', engine_name
            USING ERRCODE = 'no_data_found';
    END IF; 
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
