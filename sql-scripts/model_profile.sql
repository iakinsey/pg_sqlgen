--------------------------------------------------------------------------------
-- Model profile
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.model_profile (
    model_name TEXT PRIMARY KEY,
    config JSONB NOT NULL
);
REVOKE ALL ON TABLE sqlgen_internal.model_profile FROM public;

--------------------------------------------------------------------------------
-- Public model profile view
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.models AS
SELECT
    model_name,
    jsonb_pretty(config) AS config
FROM sqlgen_internal.model_profile;

--------------------------------------------------------------------------------
-- Get model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_model(n TEXT)
RETURNS sqlgen_internal.MODEL_PROFILE
LANGUAGE plpgsql
STRICT
AS $$
DECLARE
    r sqlgen_internal.model_profile%ROWTYPE;
BEGIN
    SELECT *
    INTO r
    FROM sqlgen_internal.model_profile mp
    WHERE mp.model_name = n;
    RETURN r;
END;
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_model FROM public;

--------------------------------------------------------------------------------
-- Create model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_model(
    model_name TEXT,
    config JSONB
)
RETURNS sqlgen_internal.MODEL_PROFILE
LANGUAGE plpgsql
STRICT
AS $$
DECLARE
    r sqlgen_internal.model_profile%ROWTYPE;
BEGIN
    INSERT INTO sqlgen_internal.model_profile(model_name, config)
    VALUES (model_name, config)
    RETURNING * INTO r;

    RETURN r;
END;
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.create_model FROM public;

--------------------------------------------------------------------------------
-- Delete model profile
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.delete_model(name TEXT)
RETURNS VOID
LANGUAGE plpgsql
STRICT
AS $$
DECLARE
    engines TEXT[];
BEGIN

    engines := sqlgen_internal.get_engines_used_by_model(name);

    IF array_length(engines, 1) IS NOT NULL THEN
        RAISE EXCEPTION
            'model % is still in use by engines: %',
            name, engines;
    END IF;

    DELETE FROM sqlgen_internal.model_profile mp
    WHERE mp.model_name = name;
END;
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.delete_model FROM public;
