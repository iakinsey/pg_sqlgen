-- Make the rust encode_text an internal functuin
ALTER FUNCTION sqlgen.encode_text(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text(TEXT, TEXT) FROM public;

-- Make the rust encode_text_batch an internal functuin
ALTER FUNCTION sqlgen.encode_text_batch(TEXT, TEXT) SET SCHEMA sqlgen_internal;
REVOKE EXECUTE ON FUNCTION sqlgen_internal.encode_text_batch(TEXT, TEXT[]) FROM public;