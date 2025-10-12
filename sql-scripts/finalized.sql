-- Make the rust encode_text an internal functuin
ALTER FUNCTION sqlgen.internal_encode_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text(TEXT, TEXT) FROM public;

CREATE FUNCTION sqlgen_internal.encode_text(model TEXT, text_value TEXT)
RETURNS VECTOR
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN internal_encode_text(model, text_value)::VECTOR;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;

-- Make the rust encode_text_batch an internal functuin
ALTER FUNCTION sqlgen.internal_encode_text_batch(TEXT, TEXT[]) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text_batch(TEXT, TEXT[]) FROM public;

CREATE FUNCTION sqlgen_internal.encode_text_batch(model TEXT, values TEXT[])
RETURNS VECTOR
LANGUAGE plpgsql
AS $$
DECLARE
    results FLOAT4[][];
BEGIN
    results := internal_encode_text_batch(model, values);
    RETURN ARRAY(SELECT v::VECTOR FROM unnest(results) AS v);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;

-- Make the rust instruct_text an internal functuin
ALTER FUNCTION sqlgen.instruct_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.instruct_text(TEXT, TEXT) FROM public;
