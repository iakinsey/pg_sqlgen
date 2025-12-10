--------------------------------------------------------------------------------
-- Create query certification table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.create_certified_queries_table(
    engine TEXT,
    vector_size INT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    vector_column_type TEXT;
BEGIN
  vector_column_type := sqlgen_internal.get_vector_column_type_sized(vector_size);

  EXECUTE format($fmt$
    CREATE TABLE sqlgen_internal.certified_queries_%I (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        language_query TEXT NOT NULL,
        sql_query TEXT NOT NULL,
        language_vector %s NOT NULL
    );

    REVOKE ALL ON TABLE sqlgen_internal.certified_queries_%I FROM PUBLIC;
  $fmt$, engine, vector_column_type, engine, engine, engine);
END
$$;

REVOKE EXECUTE
ON FUNCTION
sqlgen_internal.create_certified_queries_table FROM public;

--------------------------------------------------------------------------------
-- Delete query certification table
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.delete_certified_queries_table(
    engine TEXT
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
    DROP TABLE IF EXISTS sqlgen_internal.certified_queries_%I;
  $fmt$, engine);
END
$$;

REVOKE EXECUTE
ON FUNCTION
sqlgen_internal.delete_certified_queries_table FROM public;

--------------------------------------------------------------------------------
-- Query certification
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.certify_query(
    engine TEXT,
    id UUID,
    language_query TEXT,
    sql_query TEXT,
    language_vector FLOAT4 []
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
DECLARE
    vector_column_type TEXT;
BEGIN
    vector_column_type := sqlgen_internal.get_vector_column_type();

    EXECUTE format($fmt$
        INSERT INTO sqlgen_internal.certified_queries_%I
            (id, language_query, sql_query, language_vector)
        VALUES ($1, $2, $3, $4::%s)
    $fmt$, engine, vector_column_type)
    USING id, language_query, sql_query, language_vector;
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.certify_query FROM public;


--------------------------------------------------------------------------------
-- Get certified queries
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.get_certified_queries(
    engine_name TEXT,
    language_query_vector FLOAT4 [],
    query_limit INT
)
RETURNS TABLE (
    language_query TEXT,
    sql_query TEXT
)
LANGUAGE plpgsql
AS $$
DECLARE
    use_vec BOOLEAN;
BEGIN
    use_vec := sqlgen_internal.use_vector_search();

    IF use_vec THEN
        RETURN QUERY EXECUTE format($fmt$
            SELECT language_query, sql_query
            FROM sqlgen_internal.certified_queries_%I
            WHERE (language_vector::VECTOR <=> $1::VECTOR) <= 0.25
            ORDER BY (language_vector::VECTOR <=> $1::VECTOR) ASC
            LIMIT $2
        $fmt$, engine_name)
        USING language_query_vector::VECTOR, query_limit;
    ELSE
        RETURN QUERY EXECUTE format($fmt$
            SELECT
                language_query,
                sql_query
            FROM (
                SELECT
                    language_query,
                    sql_query,
                    sqlgen.cosine_distance(language_vector, $1) AS dist
                FROM
                    sqlgen_internal.certified_queries_%I
            ) t
            WHERE
                dist <= 0.25
            ORDER BY
                dist ASC
            LIMIT $2
        $fmt$, engine_name)
        USING language_query_vector, query_limit;
    END IF;
END
$$;


REVOKE EXECUTE ON FUNCTION sqlgen_internal.get_certified_queries FROM public;

--------------------------------------------------------------------------------
-- Delete certified query
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen_internal.decertify_query(
    engine TEXT,
    id UUID
)
RETURNS VOID
LANGUAGE plpgsql
AS $$
BEGIN
  EXECUTE format($fmt$
    DELETE FROM sqlgen_internal.certified_queries_%I WHERE id = $1
  $fmt$, engine)
  USING id;
END
$$;

REVOKE EXECUTE ON FUNCTION sqlgen_internal.decertify_query FROM public;
