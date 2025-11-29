pg_sqlgen
=========

Text-to-SQL extension for Postgres.

Connect your database to AI models. Generate queries from natural language
(text-to-sql) or explanations on how queries work (sql-to-text).

As an extension, _pg_sqlgen_ provides a number of advantages over other
text-to-sql solutions, including:
- Performance, direct access to [SPI](https://www.postgresql.org/docs/current/spi.html) eliminates latency concerns.
- Security and data locality, no data is sent over a network or to an external service.
- Authorization, everything lives within Postgres' permission model instead of an external service.
- Consistency, real-time RAG metadata maintenance via [triggers](https://www.postgresql.org/docs/current/sql-createtrigger.html).

## Installation

TODO

## Setup Development Environment

First, initialize pgrx.

```bash
$ cargo install cargo-pgrx
$ cargo pgrx init --pg18 download
$ export PG_CONFIG=$(cargo pgrx info pg-config 18)
```

Clone pgvector and set it up

```bash
$ git clone https://github.com/pgvector/pgvector.git
$ cd pgvector
$ make clean
$ make
$ make install
```

Clone pg_sqlgen and run it

```bash
$ git clone https://github.com/iakinsey/pg_sqlgen.git
$ cd pg_sqlgen
$ cargo pgrx run
```

### Running tests

#### Run tests

```bash
$ cargo pgrx test
```

#### Regression tests

```bash
$ cargo pgrx regress
```

### Linting SQL

__pg_sqlgen__ uses [sqlfluff](https://www.sqlfluff.com/) for linting and formatting. Make sure it's installed first.

To validate SQL syntax, run:

```bash
cd sql-scripts
sqlfluff fix .
```

### Building

TODO

## Basic usage

### Setup

```sql
-- Create the extension
CREATE EXTENSION sqlgen CASCADE;

-- First add a language model, for this example OpenAI models are used
SELECT sqlgen.add_model('openai_chat', sqlgen.openai_completions_config(api_key => 'sk-proj-12345'));

-- Then add an encoding model
SELECT sqlgen.add_model('openai_encode', sqlgen.openai_embeddings_config(api_key => 'sk-proj-12345'));

-- Then create an engine
SELECT sqlgen.create_engine('example_engine', 'openai_chat', 'openai_encode');

-- Optionally set the engine as a default engine
SELECT sqlgen.set_default_engine('example_engine');
```

### Usage

```sql
-- Generate SQL.
SELECT sqlgen.generate('Top 10 customers by annual spend in 2024');

-- Explain what a query does.
SELECT sqlgen.explain_query('SELECT * from customers');

-- Generate/explain if no default engine is set.
SELECT sqlgen.generate('Top 10 customers by annual spend in 2024', 'example_engine');
SELECT sqlgen.explain_query('SELECT * from customers', 'example_engine');
```

### Executing queries

```sql
BEGIN;
    SELECT sqlgen.query('Get highest paying customer');
    FETCH ALL FROM sqlgen_query;
COMMIT;
```