<h1>Generation</h1>

Functions related to text generation.

---

## generate

Generates an SQL statement from a natural language query.

### Parameters

<div style="font-size: 110%">
<code>user_query</code>
</div>

- __Description__: Natural language query
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>engine</code>
</div>

- __Description__: Engine to generate query with. If `NULL`, defaults to value
set via `set_default_engine`.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
-- Without default engine
SELECT sqlgen.generate('Get top 10 customers by monthly spend');

-- With default engine
SELECT sqlgen.generate('Get top 10 customers by monthly spend', 'my_engine');
```

---

## explain_query

Explains what a query does and how it works in natural language.

### Parameters

<div style="font-size: 110%">
<code>user_query</code>
</div>

- __Description__: SQL query
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>engine</code>
</div>

- __Description__: Engine to generate query with. If `NULL`, defaults to value
set via `set_default_engine`.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
-- Without default engine
SELECT sqlgen.explain_query('SELECT name, price FROM inventory');

-- Without default engine
SELECT sqlgen.explain_query('SELECT name, price FROM inventory' 'my_engine');
```

