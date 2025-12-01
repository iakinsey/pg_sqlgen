<h1>Examples</h1>

## Setup with OpenAI

OpenAI keys can be generated [here](https://platform.openai.com/api-keys).

```sql
-- Create the extension.
CREATE EXTENSION sqlgen CASCADE;

-- Create an instruct model.
SELECT sqlgen.add_model(
    'openai_chat',
    sqlgen.openai_completions_config(api_key => '<your OpenAI API key>')
);

-- Create an encoding model.
SELECT sqlgen.add_model(
    'openai_embed',
    sqlgen.openai_embed_config(api_key => '<your OpenAI API key>')
);

-- Create an engine
SELECT sqlgen.create_engine('example_engine', 'openai_chat', 'openai_embed');

-- Set engine as default
SELECT sqlgen.set_default_engine('example_engine');

-- Execute a generated query
BEGIN;
    SELECT sqlgen.query('Get highest paying customer');
    FETCH ALL FROM sqlgen_query;
COMMIT;
```

## Setup with Ollama

Set up [nomic-embed-text](ollama.com/library/nomic-embed-text) and
[llama3.1](https://ollama.com/library/llama3.1) with Ollama.

Start Ollama in a separate console, or via the app.

```bash
$ ollama serve
```

Set up [nomic-embed-text](ollama.com/library/nomic-embed-text) and
[llama3.1](https://ollama.com/library/llama3.1).

```bash
ollama pull nomic-embed-text
ollama pull llama3.1
```

Now setup the database extension.

```sql
-- Create the extension.
CREATE EXTENSION sqlgen CASCADE;

-- Create an instruct model.
SELECT sqlgen.add_model(
    'ollama_instruct',
    sqlgen.ollama_config(model_name => 'llama3.1')
);

-- Create an encoding model.
SELECT sqlgen.add_model(
    'ollama_embed',
    sqlgen.openai_embeddings_config(model_name => 'nomic-embed-text')
);

-- Create an engine
SELECT sqlgen.create_engine('example_engine', 'ollama_instruct', 'ollama_embed');

-- Set engine as default
SELECT sqlgen.set_default_engine('example_engine');

-- Execute a generated query
BEGIN;
    SELECT sqlgen.query('Get highest paying customer');
    FETCH ALL FROM sqlgen_query;
COMMIT;
```