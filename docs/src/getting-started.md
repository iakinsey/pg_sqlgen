# Getting Started

## Installation

### Linux

Packages on the _x86_64_ platform are available on the following distributions:
- Debian
- Ubuntu
- Fedora
- RHEL

Download pakages from [releases page](https://github.com/iakinsey/pg_sqlgen/releases/).

For Debian/Ubuntu:

```bash
$ sudo dpkg -i pg-sqlgen_<release-version>_amd64.deb
```

For RHEL/Fedora

```bash
$ dnf -y install pg_sqlgen-<release-version>-1.x86_64.rpm
```

### From source (OSX and other Linux distributions)

This assumes you already have PostgreSQL installed.

Follow the instructions in __[Setup pgvector](development.md#setup-pgvector)__ before continuing.

```bash
$ cargo pgrx init --pg-config "$(which pg_config)"

# This may require root access
$ cargo pgrx install --pg-config "$(which pg_config)" --release
```

### Building packages

Run this command to build both deb and rpm packages, producing artifacts in the repository’s root directory
([rpm](https://rpm.org/) and [fpm](https://github.com/jordansissel/fpm) is required).

```bash
make package
```

This will create installation packages in the repository's root directory.

## Create the extension

Once the package has been installed to your PostgreSQL instance, run the
following command.

```sql
CREATE EXTENSION sqlgen CASCADE;
```

## Remove the extension

Removing the extension requires the `CASCADE` parameter.

```sql
DROP EXTENSION sqlgen CASCADE;
```