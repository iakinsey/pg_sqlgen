-- Create model
SELECT sqlgen.add_model('StubModel', sqlgen.stub_config('SELECT 1', ARRAY[1.1, 1.2, 2.3, 3.4]::real[]));

-- Verify model exists
SELECT * FROM sqlgen.models;

-- Create engine
SELECT sqlgen.create_engine('StubEngine', 'StubModel', 'StubModel');

-- Verify engine exists
SELECT * FROM sqlgen.engines;

-- Set default engine
SELECT sqlgen.set_default_engine('StubEngine');

-- Execute query via cursor with default engine
SELECT sqlgen.generate('A default stub query');

-- Execute query via cursor with engine parameter
SELECT sqlgen.generate('Another stub query', 'StubEngine');

-- Delete engine
SELECT sqlgen.remove_engine('StubEngine');

-- Verify engine no longer exists
SELECT * FROM sqlgen.engines;

-- Delete model
SELECT sqlgen.delete_model('StubModel');

-- Verify model no longer exists
SELECT * FROM sqlgen.models;