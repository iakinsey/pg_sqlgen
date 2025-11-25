--------------------------------------------------------------------------------
-- Stub config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.stub_config(
    instruct_output TEXT  DEFAULT NULL,
    encode_output   REAL[] DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'Stub',
        'config', jsonb_build_object(
            'instruct_output', instruct_output,
            'encode_output',   encode_output
        )
    )
)::TEXT;
    $$;

--------------------------------------------------------------------------------
-- OpenAI completions config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.openai_completions_config(
    url                TEXT DEFAULT NULL,
    model              TEXT DEFAULT NULL,
    authorization_type TEXT DEFAULT NULL,
    api_key            TEXT DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'OpenAICompletions',
        'config', jsonb_build_object(
            'url',                url,
            'model',              model,
            'authorization_type', authorization_type,
            'api_key',            api_key
        )
    )
)::TEXT;
$$;

--------------------------------------------------------------------------------
-- OpenAI embeddings config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.openai_embeddings_config(
    url                TEXT DEFAULT NULL,
    model              TEXT DEFAULT NULL,
    authorization_type TEXT DEFAULT NULL,
    api_key            TEXT DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'OpenAIEmbeddings',
        'config', jsonb_build_object(
            'url',                url,
            'model',              model,
            'authorization_type', authorization_type,
            'api_key',            api_key
        )
    )
)::TEXT;
$$;

--------------------------------------------------------------------------------
-- Ollama config
--------------------------------------------------------------------------------

CREATE OR REPLACE FUNCTION sqlgen.ollama_config(
    host                TEXT DEFAULT NULL,
    model_name          TEXT,
    use_https           BOOLEAN DEFAULT NULL,
    request_batch_size  INTEGER DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'Ollama',
        'config', jsonb_build_object(
            'host',               host,
            'model_name',         model_name,
            'use_https',          use_https,
            'request_batch_size', request_batch_size
        )
    )
)::TEXT;
$$;
