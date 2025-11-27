# Development

## Setup

`pg_sqlgen` operates as a regular Rust codebase. It can be managed via normal
`cargo` and `cargo pgrx` commands.

### Initialize pgrx

#### Option 1 [Recommended] - Downloaded PostgreSQL instance with pgrx

Install `cargo-pgrx` and set up a PostgreSQL instance.

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

The `pgvector` extension is a required dependency for `pg_sqlgen`. It must be
installed manually.

Note: This step is not necessary if `pgvector` is already installed on an
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