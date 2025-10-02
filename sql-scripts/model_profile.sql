--------------------------------------------------------------------------------
-- Model profile
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.model_profile (
    model_name TEXT PRIMARY KEY,
    config JSONB NOT NULL
);
REVOKE ALL ON TABLE sqlgen_internal.model_profile FROM PUBLIC;

--------------------------------------------------------------------------------
-- Public model profile view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.models {
    SELECT
        model_name,
        jsonb_pretty(config) AS config
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

CREATE OR REPLACE FUNCTION sqlgen.create_model(model_name TEXT, driver_name TEXT, config JSONB)
RETURNS sqlgen_internal.model_profile
LANGUAGE plpgsql
STRICT
AS $$
BEGIN
    RETURN QUERY
    INSERT INTO sqlgen_internal.model_profile(model_name, driver_name, config)
    VALUES (model_name, driver_name, config)
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