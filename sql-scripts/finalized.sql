-- Make the rust encode_text an internal function
ALTER FUNCTION sqlgen.internal_encode_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text(TEXT, TEXT) FROM public;

CREATE OR REPLACE FUNCTION sqlgen_internal.encode_text(model TEXT, text_value TEXT)
RETURNS vector
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN sqlgen_internal.internal_encode_text(model, text_value)::vector;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;

-- Make the rust encode_text_batch an internal function
ALTER FUNCTION sqlgen.internal_encode_text_batch(TEXT, TEXT[]) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text_batch(TEXT, TEXT[]) FROM public;

CREATE OR REPLACE FUNCTION sqlgen_internal.encode_text_batch(model TEXT, text_values TEXT[])
RETURNS vector[]
LANGUAGE plpgsql
AS $$
DECLARE
    results float4[][];
BEGIN
    results := sqlgen_internal.internal_encode_text_batch(model, text_values);
    RETURN ARRAY(SELECT (v)::vector FROM unnest(results) AS v);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text_batch(TEXT, TEXT[]) FROM public;

-- Make the rust instruct_text an internal function
ALTER FUNCTION sqlgen.instruct_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.instruct_text(TEXT, TEXT) FROM public;