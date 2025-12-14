-- Setup extension
DROP EXTENSION IF EXISTS sqlgen CASCADE;
CREATE EXTENSION sqlgen;
DROP TABLE IF EXISTS s;
DROP TABLE IF EXISTS t;

-- Create model.
SELECT sqlgen.add_model(
    'stub_model',
    sqlgen.stub_config(
        '{"query": "SELECT 1;"}',
        ARRAY[1.1, 1.2, 2.3, 3.4]::REAL []
    )
);

-- Verify model exists.
SELECT * FROM sqlgen.models; -- noqa: AM04

-- Create a table before creating the engine
CREATE TABLE s (x INT);

-- Create engine.
SELECT sqlgen.create_engine(
    name => 'stub_engine',
    instruct_model => 'stub_model',
    encoder_model => 'stub_model',
    table_filter_type => 'quick'
);

-- Verify engine exists.
SELECT * FROM sqlgen.engines; -- noqa: AM04

-- Set default engine.
SELECT sqlgen.set_default_engine('stub_engine');

-- Create a new table to verify triggers don't fail.
CREATE TABLE t (x INT);

-- Alter the table to verify triggers don't fail.
ALTER TABLE t ADD COLUMN y INT;

-- Add a new comment to verify triggers don't fail.
COMMENT ON TABLE t IS 'test';

-- Generate query with default engine.
SELECT sqlgen.generate('A default stub query');

-- Generate query with engine parameter.
SELECT sqlgen.generate('Another stub query', 'stub_engine');

-- Fail to certify a query
SELECT sqlgen.certify_query(
    'Invalid query',
    'SELECT asd123'
);

-- Create and delete certified query
SELECT sqlgen.decertify_query(
    (SELECT sqlgen.certify_query('Basic query', 'SELECT 2'))
);

-- Delete engine.
SELECT sqlgen.remove_engine('stub_engine');

-- Verify engine no longer exists.
SELECT * FROM sqlgen.engines; -- noqa: AM04

-- Delete model.
SELECT sqlgen.remove_model('stub_model');

-- Verify model no longer exists.
SELECT * FROM sqlgen.models; -- noqa: AM04

-- Drop the table to verify triggers don't fail.
DROP TABLE t;
