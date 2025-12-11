--------------------------------------------------------------------------------
-- Get vector column type
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_vector_column_type()
RETURNS TEXT
LANGUAGE plpgsql
AS $$
DECLARE
    impl TEXT := sqlgen_internal.get_config_value('vector_column_impl');
BEGIN
    IF impl IS NULL OR impl = 'default' THEN
        RETURN 'FLOAT4[]';
    ELSIF impl = 'pgvector' THEN
        RETURN 'VECTOR';
    END IF;

    RETURN 'FLOAT4[]';
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_vector_column_type FROM public;


--------------------------------------------------------------------------------
-- Get vector column type sized
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_vector_column_type_sized(len INT)
RETURNS TEXT
LANGUAGE plpgsql
AS $$
DECLARE
    impl TEXT := sqlgen_internal.get_config_value('vector_column_impl');
BEGIN
    IF impl IS NULL OR impl = 'default' THEN
        RETURN 'FLOAT4[]';
    ELSIF impl = 'pgvector' THEN
        RETURN format('VECTOR(%s)', len);
    END IF;

    RETURN 'FLOAT4[]';
END
$$;

REVOKE
EXECUTE
ON FUNCTION sqlgen_internal.get_vector_column_type_sized FROM public;


--------------------------------------------------------------------------------
-- Use vector search
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.use_vector_search()
RETURNS BOOLEAN
LANGUAGE plpgsql
AS $$
DECLARE
    impl TEXT := sqlgen_internal.get_config_value('vector_column_impl');
BEGIN
    IF impl IS NULL OR impl = 'default' THEN
        RETURN FALSE;
    ELSIF impl = 'pgvector' THEN
        RETURN TRUE;
    END IF;

    RETURN FALSE;
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.use_vector_search() FROM public;
