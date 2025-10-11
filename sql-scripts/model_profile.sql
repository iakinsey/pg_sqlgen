--------------------------------------------------------------------------------
-- Model profile
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.model_profile (
    model_name TEXT PRIMARY KEY,
    db_schema TEXT NOT NULL,
    config JSONB NOT NULL,
    generate_prompt TEXT,
    filter_prompt TEXT
);
REVOKE ALL ON TABLE sqlgen_internal.model_profile FROM PUBLIC;

--------------------------------------------------------------------------------
-- Public model profile view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.models {
    SELECT
        model_name AS model_name,
        db_schema AS db_schema,
        jsonb_pretty(config) AS config,
        generate_prompt AS generate_prompt,
        filter_prompt AS filter_prompt
    FROM sqlgen_internal.model_profile
}

--------------------------------------------------------------------------------
-- Get model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.get_model(model_name TEXT)
RETURNS sqlgen_internal.model_profile
LANGUAGE plpgsql 
STRICT
AS $$
BEGIN
    RETURN QUERY
    SELECT *
    FROM sqlgen_internal.model_profile mp
    WHERE mp.model_name = model_name;
END;
$$;

--------------------------------------------------------------------------------
-- Create model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.create_model(
    model_name TEXT,
    db_schema TEXT,
    config JSONB,
    generate_prompt TEXT DEFAULT NULL,
    filter_prompt TEXT DEFAULT NULL
)
RETURNS sqlgen_internal.model_profile
LANGUAGE plpgsql
STRICT
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO sqlgen_internal.model_profile(model_name, db_schema, config, generate_prompt, filter_prompt)
    VALUES (model_name, db_schema, config, generate_prompt, filter_prompt)
    RETURNING *;
END;
$$;

--------------------------------------------------------------------------------
-- Delete model profile
-- TODO handle cases when model is already in use
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.delete_model(model_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
STRICT
AS $$
BEGIN
    DELETE FROM sqlgen_internal.model_profile mp
    WHERE mp.model_name = model_name;
END;
$$;