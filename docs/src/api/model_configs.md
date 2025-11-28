<h1>Model Configs</h1>

Model configs define how models interact with the extension. They are used as
parameters for [`add_model`](./model.md#add_model).

## ollama_config

Configuration for models that run on [Ollama](https://ollama.com/).

### Capabilities
- Instruct
- Encoding

### Parameters

<div style="font-size: 110%">
<code>host</code>
</div>

- __Description__: [Ollama API](https://docs.ollama.com/api/introduction) host.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `localhost:11434`

<div style="font-size: 110%">
<code>model_name</code>
</div>

- __Description__: Name of model.
- __Type__: `TEXT`
- __Optional__: `false`

<div style="font-size: 110%">
<code>use_https</code>
</div>

- __Description__: Indicates if the RPC request should use SSL.
- __Type__: `BOOLEAN`
- __Optional__: `true`
- __Default__: `false`

<div style="font-size: 110%">
<code>request_batch_size</code>
</div>

- __Description__: Number of requests that can be sent to the server in
parallel.
- __Type__: `INTEGER`
- __Optional__: `true`
- __Default__: `64`

### Example

```sql
SELECT sqlgen.add_model(
    'ollama_model',
    sqlgen.ollama_config(
        host => 'localhost:11434',
        model_name => 'llama3',
        use_https => false,
        request_batch_size => 16
    )
);
```

---

## openai_completions_config

Configuration for models that use the [OpenAI Completions API](https://platform.openai.com/docs/guides/completions).

### Capabilities
- Instruct

### Parameters

<div style="font-size: 110%">
<code>url</code>
</div>

- __Description__: Completions API host.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `https://api.openai.com/v1/chat/completions`

<div style="font-size: 110%">
<code>model</code>
</div>

- __Description__: Name of model.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `gpt-5.1`

<div style="font-size: 110%">
<code>authorization_type</code>
</div>

- __Description__: Value to set in `Authorization` header before the api key.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `Bearer`

<div style="font-size: 110%">
<code>api_key</code>
</div>

- __Description__: API key to be used in the `Authorization` header. Header is
omitted if this value is not set.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

```sql
    SELECT sqlgen.add_model(
        'openai_chat_model',
        sqlgen.openai_completions_config(
            url => 'https://api.openai.com/v1/chat/completions',
            model => 'gpt-4.1',
            authorization_type => 'Bearer',
            api_key => 'sk-proj-1234567'
        )
    );
```

## openai_embeddings_config 

Configuration for models that use the [OpenAI Embeddings API](https://platform.openai.com/docs/api-reference/embeddings)

### Capabilities
- Encoding

### Parameters

<div style="font-size: 110%">
<code>url</code>
</div>

- __Description__: Embeddings API host.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `https://api.openai.com/v1/embeddings`

<div style="font-size: 110%">
<code>model</code>
</div>

- __Description__: Name of model.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `text-embedding-3-small`

<div style="font-size: 110%">
<code>authorization_type</code>
</div>

- __Description__: Value to set in `Authorization` header before the api key.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `Bearer`

<div style="font-size: 110%">
<code>api_key</code>
</div>

- __Description__: API key to be used in the `Authorization` header. Header is
omitted if this value is not set.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `NULL`

```sql
    SELECT sqlgen.add_model(
        'openai_embed_model',
        sqlgen.openai_embeddings_config(
            url => 'https://api.openai.com/v1/embeddings',
            model => 'text-embedding-3-small',
            authorization_type => 'Bearer',
            api_key => 'sk-proj-1234567'
        )
    );
```
---

## stub_config

Outputs exactly what is provided in parameters. Used for testing purposes.

### Capabilities
- Instruct
- Encoding

### Parameters

<div style="font-size: 110%">
<code>instruct_output</code>
</div>

- __Description__: Output when used as an instruct model.
- __Type__: `TEXT`
- __Optional__: `true`
- __Default__: `stub`

<div style="font-size: 110%">
<code>encode_output</code>
</div>

- __Description__: Output when used as an encoder model.
- __Type__: `REAL[]`
- __Optional__: `true`
- __Default__: `ARRAY[0.0, 0.0, 0.0]::REAL[]`

### Example

```sql
    SELECT sqlgen.add_model(
        'test_stub',
        sqlgen.stub_config(
            instruct_output => 'Hello world!',
            encode_output => ARRAY[1.0, 2.5, 4.1]::REAL[]
        );
    );
```