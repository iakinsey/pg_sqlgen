<h1>Vector</h1>

Functions related to vector backend management.

## set_vector_backend {#set-vector-backend}

Toggles the vector implementation used internally.
See [Internals - Vector Backend](../internals.md#vector-backends)
for more details.

### Returns

- __Type__: `TEXT`

### Parameters

<div style="font-size: 110%">
<code>backend</code>
</div>

- __Description__: Specifies which backend to use. If omitted, it switches to the unused backend.
  Can either be `pgvector` or `default`.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

### Example

```sql
-- Switch to default backend
SELECT sqlgen.set_vector_backend('default');

-- Switch to pgvector backend
SELECT sqlgen.set_vector_backend('pgvector');

-- Switch to unused baclemd
SELECT sqlgen.set_vector_backend();
```