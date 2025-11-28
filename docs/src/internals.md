# Internals

These Internal mechanisms may inform how to configure and tune the extension.

## RAG workflow {#rag-workflow}

The rag workflow for text-to-sql generation works as follows:
- Get DDLs relevant to the user query. The process will vary depending on the
engine's `table_filter_type` value:
    - `smart` - The instruct model is asked to filter for DDLs relevant to the
    user query.
    - `quick` - DDLs are filtered via cosine similarity through pgvector.
- Provide the user query, DDLs, and relevant instructions to the instruct model.
- The resulting query run against a `PREPARE` statement to check if it's valid.
    - If it's valid, return the query to the user.
    - If the query is not valid, the instruct model is asked to fix the query. 
    The new query is returned to the user.

## Metadata triggers {#metadata-triggers}

Each engine creates triggers for these operations in its target schema:
- `CREATE TABLE`
- `ALTER TABLE`
- `DROP TABLE`
- `COMMENT ON`

When new tables or columns are added, the encoder model generates semantic
vectors. These triggers are deleted when the engine is removed.

## Prompt templates {#prompt-templates}

- All prompt templates are rendered using
  [tera](https://docs.rs/tera/latest/tera/).
- Templates require specific blocks to render correctly, some block values can
  be specified as parameters when creating an engine.
- Default prompts can be found [here](https://github.com/iakinsey/pg_sqlgen/blob/main/src/types/structs/engine.rs).

### system_prompt_template

System prompt containing high-level instructions for text-to-sql generation.

#### Blocks
- __output_format_description__ - Text instructions on how the model should output
structured data to be consumed by the engine.

### user_prompt_template

User prompt containing specific instructions for text-to-sql generation.

#### Blocks
- __user_query__ - Natural language query to be converted 
- __relevant_ddls_block__ - DDL list with optional surrounding text, see [relevant_ddls_template](#relevant_ddls_template)
- __similar_queries_block__ - List of SQL queries similar to the user query, see [similar_queries_template](#similar_queries_template).

### relevant_ddls_template

Block used to present DDL listing when generating SQL from text.

#### Blocks
- __relevant_ddls__ - Raw list of DDLs relevant to the query

### similar_queries_template

Block used to present similar queries when generating SQL from text. Currently
unused.

### filter_ddls_template

User prompt containing specific instructions for filtering DDLs. Used when
`table_filter_type` is set to `smart`.

#### Blocks
- __user_query__ - User query used to filter DDLs with.
- __relevant_ddls__ - List of DDLs to filter.

### syntax_correction_template

User prompt containing specific instructions for correcting an erroneous query.

#### Blocks
- __query__ - The erroneous query.
- __error_message__ - Error related to the query.

### explain_query_template

User prompt containing specific instructions for explaining what a query does
and how it works.

#### Blocks
- __output_format_description__ - Text instructions on how the model should output
structured data to be consumed by the engine.
- __sql_query__ - The query to be explained
- __explain__ - Contains the output of `EXPLAIN` against the SQL query.