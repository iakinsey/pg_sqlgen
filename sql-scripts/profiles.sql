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

CREATE OR REPLACE FUNCTION sqlgen.local_phi_config(
    model_name         TEXT    DEFAULT NULL,
    revision           TEXT    DEFAULT NULL,
    compute_device     TEXT    DEFAULT NULL,
    tokenizer_filename TEXT    DEFAULT NULL,
    config_filename    TEXT    DEFAULT NULL,
    weights_filenames  TEXT[]  DEFAULT NULL,
    sample_len         INTEGER DEFAULT NULL,
    repeat_penalty     TEXT    DEFAULT NULL,
    repeat_last_n      INTEGER DEFAULT NULL,
    seed               BIGINT  DEFAULT NULL,
    temperature        TEXT    DEFAULT NULL,
    top_p              TEXT    DEFAULT NULL,
    data_type          TEXT    DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'LocalPhi',
        'config', jsonb_build_object(
            'model_name',         model_name,
            'revision',           revision,
            'compute_device',     compute_device,
            'tokenizer_filename', tokenizer_filename,
            'config_filename',    config_filename,
            'weights_filenames',  weights_filenames,
            'sample_len',         sample_len,
            'repeat_penalty',     repeat_penalty,
            'repeat_last_n',      repeat_last_n,
            'seed',               seed,
            'temperature',        temperature,
            'top_p',              top_p,
            'data_type',          data_type
        )
    )
)::TEXT;
$$;

CREATE OR REPLACE FUNCTION sqlgen.local_bert_config(
    model_name         TEXT DEFAULT NULL,
    revision           TEXT DEFAULT NULL,
    config_filename    TEXT DEFAULT NULL,
    tokenizer_filename TEXT DEFAULT NULL,
    weights_filename   TEXT DEFAULT NULL,
    compute_device     TEXT DEFAULT NULL
)
RETURNS TEXT
LANGUAGE SQL
AS $$
SELECT jsonb_strip_nulls(
    jsonb_build_object(
        'type',   'LocalBert',
        'config', jsonb_build_object(
            'model_name',         model_name,
            'revision',           revision,
            'config_filename',    config_filename,
            'tokenizer_filename', tokenizer_filename,
            'weights_filename',   weights_filename,
            'compute_device',     compute_device
        )
    )
)::TEXT;
$$;
