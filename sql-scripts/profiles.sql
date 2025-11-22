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
