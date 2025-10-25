--------------------------------------------------------------------------------
-- Table
--------------------------------------------------------------------------------

CREATE TABLE sqlgen_internal.config (
    "key" TEXT NOT NULL PRIMARY KEY,
    "value" TEXT NOT NULL
);
REVOKE ALL ON TABLE sqlgen_internal.config FROM public;

--------------------------------------------------------------------------------
-- View
--------------------------------------------------------------------------------

CREATE OR REPLACE VIEW sqlgen.config AS
    SELECT "key", "value" FROM sqlgen_internal.config;

--------------------------------------------------------------------------------
-- Get config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_config_value(k TEXT)
RETURNS TEXT
LANGUAGE plpgsql
AS $$
DECLARE
    result TEXT;
BEGIN
    SELECT "value" INTO result
    FROM sqlgen_internal.config
    WHERE key = k;

    RETURN result;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_config_value(TEXT) FROM public;

--------------------------------------------------------------------------------
-- Set config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.set_config_value(k TEXT, v TEXT)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
    INSERT INTO sqlgen_internal.config(key, value)
    VALUES (k, v)
    ON CONFLICT (key) DO UPDATE
    SET value = EXCLUDED.value;
END;
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.set_config_value(TEXT, TEXT) FROM public;