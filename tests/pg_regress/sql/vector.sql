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
    'stub_engine_1',
    'stub_model_1',
    'stub_model_1',
    'quick'
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

-- Change vector impl to VECTOR
SELECT sqlgen.set_vector_backend();

-- Execute query
SELECT sqlgen.generate('A default stub query', 'stub_engine_1');
