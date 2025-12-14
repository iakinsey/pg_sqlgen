-- Setup extension
DROP EXTENSION IF EXISTS sqlgen CASCADE;
DROP EXTENSION IF EXISTS vector;
CREATE EXTENSION sqlgen;
DROP TABLE IF EXISTS v;

-- Create model.
SELECT sqlgen.add_model(
    'stub_model_1',
    sqlgen.stub_config(
        '{"query": "SELECT 1;"}',
        ARRAY[1.1, 1.2, 2.3, 3.4]::REAL []
    )
);

-- Create engine.
SELECT sqlgen.create_engine(
    name => 'stub_engine_1',
    instruct_model => 'stub_model_1',
    encoder_model => 'stub_model_1',
    table_filter_type => 'quick'
);

-- Create a new table to verify triggers don't fail.
CREATE TABLE v (x INT);

-- Add a new comment 
COMMENT ON TABLE v IS 'test';

-- Fail to change vector impl
SELECT sqlgen.set_vector_backend();

-- Create extension then try again
CREATE EXTENSION vector;

-- Change vector impl to VECTOR
SELECT sqlgen.set_vector_backend();

-- Execute query
SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
DO $$
BEGIN
    PERFORM sqlgen.certify_query('example query', 'SELECT 1', 'stub_engine_1');
END;
$$;

-- Change vector impl to default
SELECT sqlgen.set_vector_backend();

-- Execute query
SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
DO $$
BEGIN
    PERFORM sqlgen.certify_query('example query', 'SELECT 1', 'stub_engine_1');
END;
$$;

-- Change to default while already default
SELECT sqlgen.set_vector_backend('default');
SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
DO $$
BEGIN
    PERFORM sqlgen.certify_query('example query', 'SELECT 1', 'stub_engine_1');
END;
$$;

-- Change to pgvector
SELECT sqlgen.set_vector_backend('pgvector');

SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
DO $$
BEGIN
    PERFORM sqlgen.certify_query('example query', 'SELECT 1', 'stub_engine_1');
END;
$$;

-- Change to default 
SELECT sqlgen.set_vector_backend('default');

SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
DO $$
BEGIN
    PERFORM sqlgen.certify_query('example query', 'SELECT 1', 'stub_engine_1');
END;
$$;
