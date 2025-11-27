# Development

## Setup

__pg_sqlgen__ operates as a regular Rust codebase. It can be managed via
`cargo` and `cargo pgrx` commands.

### Initialize pgrx {#initialize-pgrx}

#### Option 1 [Recommended] - Downloaded PostgreSQL instance with pgrx

Install __cargo-pgrx__ and set up a PostgreSQL instance.

```bash
$ cargo install cargo-pgrx
$ cargo pgrx init --pg18 download
$ export PG_CONFIG=$(cargo pgrx info pg-config 18)
```

#### Option 2 - Use an existing PostgreSQL instance

Do this if you plan on using an existing instance of PostgreSQL for development.

```bash
$ cargo install cargo-pgrx
$ cargo pgrx init --pg18 /path/to/pg_config
$ export PG_CONFIG=/path/to/pg_config
```

### Setup pgvector

The __pgvector__ extension is a required dependency for __pg_sqlgen__. It must be
installed manually.

Note: This step is not necessary if __pgvector__ is already installed on an
existing PostgreSQL instance.

```bash
$ git clone https://github.com/pgvector/pgvector.git
$ cd pgvector
$ make clean
$ make
$ make install
```

## Testing

Testing is done through normal `cargo pgrx` methods.

### Unit tests

```bash
$ cargo pgrx test
```

### Regression tests

```bash
$ cargo pgrx regress
```