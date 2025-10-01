--------------------------------------------------------------------------------
-- Model profile
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.model_profile (
    model_name TEXT PRIMARY KEY,
    driver_name TEXT NOT NULL,
    config JSONB NOT NULL
);
REVOKE ALL ON TABLE sqlgen_internal.model_profile FROM PUBLIC;

--------------------------------------------------------------------------------
-- Public model profile view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.model_profiles {
    SELECT
        model_name,
        driver_name,
        jsonb_pretty(config) AS config
    FROM sqlgen_internal.model_profile
}

--------------------------------------------------------------------------------
-- Get model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_model_profile(model_name TEXT)
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
REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_model_profile(TEXT) FROM public;

--------------------------------------------------------------------------------
-- Create model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_model_profile(model_name TEXT, driver_name TEXT, config JSONB)
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
REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_model_profile(TEXT, TEXT, JSONB) FROM public;

--------------------------------------------------------------------------------
-- Delete model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.delete_model_profile(model_name TEXT)
RETURNS VOID
LANGUAGE plpgsql
STRICT
AS $$
BEGIN
    DELETE FROM sqlgen_internal.model_profile mp
    WHERE mp.model_name = model_name;
END;
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.delete_model_profile(TEXT) FROM public;