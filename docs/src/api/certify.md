<h1>Query Certification</h1>

Query certification is a mechanism for storing queries that
users consider ideal examples. These certified queries serve
as reference specimens that language models can learn from,
improving the accuracy of generated SQL.

A maximum of three certified queries with a cosine similarity
of ≥ 0.75 are injected into the prompt.

---

## certify_query

Certifies a functional query so that it can be provided to
prompts to boost accuracy. Returns a unique identifier for
the certified query.

### Returns

- __Type__: `TEXT`
- __Description__: A UUID identifying the query certification instance.

### Parameters

<div style="font-size: 110%">
<code>language_query</code>
</div>

- __Description__: Natural language query that accurately reflects `sql_query`.
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>sql_query</code>
</div>

- __Description__: SQL query that accurately reprsents the `language_query`.
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>engine</code>
</div>

- __Description__: Engine to set query for. If `NULL`, defaults to value
set via `set_default_engine`.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
-- Without default engine
SELECT sqlgen.certify_query(
    'SELECT name, price FROM inventory',
    'Get name and price from inventory'
);

-- Without default engine
SELECT sqlgen.certify_query(
    'SELECT name, price FROM inventory',
    'Get name and price from inventory',
    'my_engine'
);
```

---

## decertify_query

Removes a certified query.

### Returns

- __Type__: `VOID`

### Parameters

<div style="font-size: 110%">
<code>id</code>
</div>

- __Description__: Identifier for certified query.
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>engine</code>
</div>

- __Description__: Engine to remove query from. If `NULL`, defaults to value
set via `set_default_engine`.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
-- Without default engine
SELECT sqlgen.decertify_query('6d6e1596-f3a8-4847-a1d8-ccb3d60fc0e6');

-- Without default engine
SELECT sqlgen.decertify_query(
    '6d6e1596-f3a8-4847-a1d8-ccb3d60fc0e6',
    'my_engine'
);
```

