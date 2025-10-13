--------------------------------------------------------------------------------
-- Text to SQL engine
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.engine (
    engine_name TEXT NOT NULL PRIMARY KEY,
    schema_name TEXT NOT NULL,
    encoder_model TEXT NOT NULL,
    instruct_model TEXT NOT NULL,
    generate_prompt TEXT,
    filter_prompt TEXT,
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
-- Create engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_engine(model_name TEXT, schema_name TEXT, instruct_model TEXT, filter_prompt TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- TODO
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_engine(TEXT, TEXT, TEXT, TEXT) FROM public;

--------------------------------------------------------------------------------
-- Remove engine
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.remove_engine(model_name TEXT, schema_name TEXT, instruct_model TEXT, filter_prompt TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    -- TODO
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_engine(TEXT, TEXT, TEXT, TEXT) FROM public;

