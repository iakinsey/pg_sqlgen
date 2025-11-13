-- Make the rust encode_text an internal function
ALTER FUNCTION internal_encode_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_encode_text(TEXT, TEXT) FROM public;

-- Make the rust get_descriptions an internal function and expose it as a public view
ALTER FUNCTION internal_get_descriptions() SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_get_descriptions() FROM public;
CREATE OR REPLACE VIEW sqlgen.model_descriptions AS SELECT * FROM sqlgen_internal.internal_get_descriptions();
--CREATE OR REPLACE VIEW sqlgen.model_descriptions AS SELECT * FROM internal_get_descriptions();


CREATE OR REPLACE FUNCTION sqlgen_internal.encode_text(engine TEXT, text_value TEXT)
RETURNS vector
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN sqlgen_internal.internal_encode_text(engine, text_value)::vector;
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;

-- Make the rust batch_text_encode an internal function
ALTER FUNCTION internal_batch_text_encode(TEXT, TEXT[]) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_batch_text_encode(TEXT, TEXT[]) FROM public;

CREATE OR REPLACE FUNCTION sqlgen_internal.batch_text_encode(model TEXT, text_values TEXT[])
RETURNS vector[]
LANGUAGE plpgsql
AS $$
DECLARE
    results float4[][];
BEGIN
    results := sqlgen_internal.batch_text_encode(model, text_values);
    RETURN ARRAY(SELECT (v)::vector FROM unnest(results) AS v);
END
$$;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.batch_text_encode(TEXT, TEXT[]) FROM public;

-- Make the rust batch_text_encode an internal function
ALTER FUNCTION internal_add_table(TEXT, TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.internal_add_table(TEXT, TEXT, TEXT) FROM public;
ALTER FUNCTION sqlgen_internal.internal_add_table(TEXT, TEXT, TEXT) RENAME TO add_table;
