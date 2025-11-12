use serde_json::to_string;

use crate::{
    stores::engine_store::EngineStore,
    types::structs::{engine::TextToSqlEngine, model_profile::ModelConfig, profiles::StubConfig},
};
use pgrx::Spi;

pub(crate) fn create_engine(
    name: &str,
    schema_name: &str,
    table_filter_type: &str,
    expected_instruct_output: &str,
) -> TextToSqlEngine {
    let encoder_model_name = "test_encoder_model_name";
    let instruct_model_name = "test_instruct_model_name";
    let expected_encoding_output = vec![0.0, 0.1, 0.2, 0.3];
    let config = StubConfig {
        instruct_output: expected_instruct_output.to_string(),
        encode_output: expected_encoding_output,
    };
    let encoder_profile = ModelConfig::Stub(config.clone());
    let instruct_profile = ModelConfig::Stub(config);
    let encoder_profile_json = to_string(&encoder_profile).unwrap();
    let instruct_profile_json = to_string(&instruct_profile).unwrap();

    Spi::run_with_args(
        "SELECT sqlgen.create_model($1, $2::JSONB);",
        &[encoder_model_name.into(), encoder_profile_json.into()],
    )
    .unwrap();

    Spi::run_with_args(
        "SELECT sqlgen.create_model($1, $2::JSONB);",
        &[instruct_model_name.into(), instruct_profile_json.into()],
    )
    .unwrap();

    EngineStore::create_engine(
        name,
        schema_name,
        encoder_model_name,
        instruct_model_name,
        table_filter_type,
        None,
        None,
        None,
        None,
        None,
    )
    .unwrap();

    EngineStore::get_engine(name).unwrap()
}

pub(crate) fn create_schema(name: &str) {
    Spi::run(
        format!(
            r#"
                CREATE SCHEMA {name};

                CREATE TABLE {name}.users (
                    user_id SERIAL PRIMARY KEY,
                    username TEXT NOT NULL UNIQUE,
                    email TEXT NOT NULL UNIQUE,
                    created_at TIMESTAMP DEFAULT NOW()
                );

                COMMENT ON COLUMN {name}.users.user_id IS 'Hello world';
                COMMENT ON COLUMN {name}.users.username IS 'Hello world';
                COMMENT ON COLUMN {name}.users.email IS 'Hello world';
                COMMENT ON COLUMN {name}.users.created_at IS 'Hello world';


                CREATE TABLE {name}.products (
                    product_id SERIAL PRIMARY KEY,
                    name TEXT NOT NULL,
                    description TEXT,
                    price NUMERIC(10, 2) NOT NULL,
                    created_at TIMESTAMP DEFAULT NOW()
                );

                COMMENT ON COLUMN {name}.products.product_id IS 'Hello world';
                COMMENT ON COLUMN {name}.products.name IS 'Hello world';
                COMMENT ON COLUMN {name}.products.description IS 'Hello world';
                COMMENT ON COLUMN {name}.products.price IS 'Hello world';
                COMMENT ON COLUMN {name}.products.created_at IS 'Hello world';


                CREATE TABLE {name}.orders (
                    order_id SERIAL PRIMARY KEY,
                    user_id INT NOT NULL REFERENCES {name}.users(user_id) ON DELETE CASCADE,
                    order_date TIMESTAMP DEFAULT NOW(),
                    total NUMERIC(10, 2) NOT NULL
                );

                COMMENT ON COLUMN {name}.orders.order_id IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.user_id IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.order_date IS 'Hello world';
                COMMENT ON COLUMN {name}.orders.total IS 'Hello world';


                CREATE TABLE {name}.order_items (
                    order_item_id SERIAL PRIMARY KEY,
                    order_id INT NOT NULL REFERENCES {name}.orders(order_id) ON DELETE CASCADE,
                    product_id INT NOT NULL REFERENCES {name}.products(product_id),
                    quantity INT NOT NULL CHECK (quantity > 0),
                    price NUMERIC(10, 2) NOT NULL
                );

                COMMENT ON COLUMN {name}.order_items.order_item_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.order_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.product_id IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.quantity IS 'Hello world';
                COMMENT ON COLUMN {name}.order_items.price IS 'Hello world';
            "#,
            name = name
        )
        .as_str(),
    )
    .unwrap();
}
