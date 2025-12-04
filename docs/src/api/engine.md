<h1>Engine</h1>

Engines contain all of the relevant prompt templates and metadata required to
generate results from [generation](./generation.md) functions. An engine must
be created in order to generate SQL.

All engine names must be in [snake case](https://en.wikipedia.org/wiki/Snake_case).

---

## engines

List of engines currently available.

### Usage

```sql
SELECT * FROM sqlgen.engines;
```

---

## create_engine

Create an engine.

### Parameters

<div style="font-size: 110%">
<code>name</code>
</div>

- __Description__: Engine name.
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>instruct_model</code>
</div>

- __Description__: Name of instruct model to use for engine (see [model](./model.md#capabilities)).
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>encoder_model</code>
</div>

- __Description__: Name of encoder model to use for engine (see [model](./model.md#capabilities)).
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>schema_name</code>
</div>

- __Description__: Name of database schema that model operates on.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: [`current_schema()`](https://www.postgresql.org/docs/7.3/functions-misc.html)

<div style="font-size: 110%">
<code>table_filter_type</code>
</div>

- __Description__: Determines how tables are filtered in the [RAG workflow](../internals.md#rag-workflow). `quick` uses [cosine similarity](https://en.wikipedia.org/wiki/Cosine_similarity), `smart` uses an LLM.
- __Type__: `TEXT`, either `quick` or `smart`
- __Optional__: `true`
- __Default__: `smart`

<div style="font-size: 110%">
<code>ddl_prompt_limit</code>
</div>

- __Description__: Determines how many DDLS are passed into `relevant_ddls_template` at a time. Useful if there are strong limits on a model's maximum token input.
- __Type__: `INT`
- __Optional__: `true`
- __Default__: `128`

<div style="font-size: 110%">
<code>error_correction_rounds</code>
</div>

- __Description__: Determines how many rounds of error correction should be made before giving up.
- __Type__: `INT`
- __Optional__: `true`
- __Default__: `3`

<div style="font-size: 110%">
<code>system_prompt_template</code>
</div>

- __Description__: System prompt used when generating SQL queries from text. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>user_prompt_template</code>
</div>

- __Description__: User prompt used when generating SQL queries from text. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>relevant_ddls_template</code>
</div>

- __Description__: Segment of `user_prompt_template` used to specify how relevant DDL's are rendered. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>similar_queries_template</code>
</div>

- __Description__: Segment of `user_prompt_template` used to specify how similar queries are rendered. Currently not in use. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>filter_ddls_template</code>
</div>

- __Description__: User prompt used when filtering DDLs with the `smart` filter. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>syntax_correction_template</code>
</div>

- __Description__: User prompt used when correcting SQL syntax. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

<div style="font-size: 110%">
<code>explain_query_template</code>
</div>

- __Description__: User prompt used when generating explanations of how queries work. See [Prompt templates](../internals.md#prompt-templates) for more information about formatting and defaults.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
SELECT create_engine('my_engine', 'my_llm', 'my_encoder');
```

---

## remove_engine

Remove an engine.

### Parameters

<div style="font-size: 110%">
<code>name</code>
</div>

- __Description__: Name of engine.
- __Type__: `TEXT`
- __Optional__: `false`

### Example

```sql
SELECT sqlgen.remove_engine('my_engine');
```

---

## set_default_engine

Sets the default engine. When [generation](./generation.md) functions are called
without an engine specified, the value set here is used.

### Parameters

<div style="font-size: 110%">
<code>engine</code>
</div>

- __Description__: Name of engine.
- __Type__: `TEXT`
- __Optional__: `false`

### Example

```sql
SELECT sqlgen.set_default_engine('my_engine');
```

---

## remove_default_engine

Removes the default engine.

### Example

```sql
SELECT sqlgen.remove_default_engine('my_engine');
```
