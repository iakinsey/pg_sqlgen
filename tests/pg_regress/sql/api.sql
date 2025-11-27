-- Create model
SELECT sqlgen.add_model('stub_model', sqlgen.stub_config('{"query": "SELECT 1;"}', ARRAY[1.1, 1.2, 2.3, 3.4]::real[]));

-- Verify model exists
SELECT * FROM sqlgen.models;

-- Create engine
SELECT sqlgen.create_engine('stub_engine', 'stub_model', 'stub_model');

-- Verify engine exists
SELECT * FROM sqlgen.engines;

-- Set default engine
SELECT sqlgen.set_default_engine('stub_engine');

-- Execute query via cursor with default engine
SELECT sqlgen.generate('A default stub query');

-- Execute query via cursor with engine parameter
SELECT sqlgen.generate('Another stub query', 'stub_engine');

-- Delete engine
SELECT sqlgen.remove_engine('stub_engine');

-- Verify engine no longer exists
SELECT * FROM sqlgen.engines;

-- Delete model
SELECT sqlgen.remove_model('stub_model');

-- Verify model no longer exists
SELECT * FROM sqlgen.models;

-- TODO add new data and make sure it reflects internals somehow