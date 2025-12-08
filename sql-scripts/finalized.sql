-- Make the rust encode_text an internal function
ALTER FUNCTION internal_encode_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text(
    TEXT, TEXT
) FROM public;

-- Expose get_descriptions as a public view
CREATE OR REPLACE VIEW sqlgen.model_descriptions AS
SELECT * -- noqa: AM04
FROM sqlgen_internal.get_descriptions();


-- Make the rust encode_text an internal function, alter return type
CREATE OR REPLACE FUNCTION sqlgen_internal.encode_text(
    engine TEXT, text_value TEXT
)
RETURNS FLOAT4 []
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN sqlgen_internal.internal_encode_text(engine, text_value);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;
