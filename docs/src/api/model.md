## Model

Functions related to model management. Models are necessary for
[engines](./engine.md) to make inference calls.

Models must support at least one of two capabilities, text encoding or chat.
For more information about what each model interface supports, See
[Model Configs](./model_configs.md) for supported interfaces.

---

## add_model

Creates a new model profile.

### Parameters

<div style="font-size: 110%">
<code>model_name</code>
</div>

- __Description__: Name of model
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>config_str</code>
</div>

- __Description__: Model configuration
- __Type__: [Model config](./model_configs.md)
- __Optional__: `false`

### Example

```sql
SELECT sqlgen.add_model(
    'openai_chat',
    sqlgen.openai_completions_config(api_key => 'sk-proj-12345')
);
```

---

## remove_model

Deletes a model profile.

### Parameters

<div style="font-size: 110%">
<code>model_name</code>
</div>

- __Description__: Name of model
- __Type__: `TEXT`
- __Optional__: `false`

### Example

```sql
SELECT sqlgen.remove_model('openai_chat');
```
