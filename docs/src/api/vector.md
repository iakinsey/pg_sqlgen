<h1>Vector</h1>

Functions related to vector backend management.

## toggle_vector_impl {#toggle-vector-impl}

Toggles the vector implementation used internally.
See [Internals - Vector Backend](../internals.md#vector-backends)
for more details.

### Returns

- __Type__: `VOID`

### Example

```sql
SELECT sqlgen.toggle_vector_impl();
```