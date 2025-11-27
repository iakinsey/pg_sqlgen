# Installation

## Via packages

TODO

## Via pgrx

Follow the instructions in __[Initialize pgrx](development.md#initialize-pgrx)__ before continuing.

```bash
cargo pgrx install
```

## Installing extension in PostgreSQL

Once the package has been installed to your PostgreSQL instance, run the
following command.

```sql
CREATE EXTENSION sqlgen CASCADE;
```