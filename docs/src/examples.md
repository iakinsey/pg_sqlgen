<h1>Examples</h1>

## Setting up a text-to-sql engine using OpenAI

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
SELECT EXECUTE sqlgen.generate('Top 10 highest grossing films.');
```

## Setting up a text-to-sql engine using Ollama

Set up [nomic-embed-text](ollama.com/library/nomic-embed-text) and
[llama3.1](https://ollama.com/library/llama3.1) with Ollama

Start Ollama in a separate console.

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
    sqlgen.ollama_config(model => 'llama3.1')
);

-- Create an encoding model.
SELECT sqlgen.add_model(
    'ollama_embed',
    sqlgen.openai_embed_config(model => 'nomic-embed-text')
);

-- Create an engine
SELECT sqlgen.create_engine('example_engine', 'ollama_instruct', 'ollama_embed');

-- Set engine as default
SELECT sqlgen.set_default_engine('example_engine');

-- Execute a generated query
SELECT EXECUTE sqlgen.generate('Top 10 highest grossing films.');

```