# Introduction

__pg_sqlgen__ is a database extension for PostgreSQL that enables text-to-sql
functionality. It is based on the [Dubo-SQL](https://arxiv.org/abs/2404.12560)
method for fine-tuned Text-to-SQL generation.

As an extension, __pg_sqlgen__ provides a number of advantages over other
text-to-sql solutions, including:
- Performance, direct access to [SPI](https://www.postgresql.org/docs/current/spi.html) eliminates latency concerns.
- Security and data locality, no data is sent over a network or to an external service.
- Authorization, everything lives within Postgres' permission model instead of an external service.
- Consistency, real-time RAG metadata maintenance via triggers.

## Features

- Simple text-to-sql generation via `sqlgen.generate()`.
- Explain what queries do and how they work via `sqlgen.explain_query()`.
- Continuous schema-change capture through extension-managed triggers.
- Support for multiple model APIs
- Support for multiple independent text-to-SQL engines with isolated RAG
  metadata.