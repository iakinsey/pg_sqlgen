use crate::{
    stores::{metadata_store::MetadataStore, model_store::ModelStore},
    types::{
        errors::SqlgenError,
        formats::ddl_output_format_schema,
        structs::{
            ddl_response::DDLResponse,
            engine::{TableFilterType, TextToSqlEngine},
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
};
use serde_json::from_str;
use tera::{Context, Tera};

pub static USER_QUERY_VAR_KEY: &str = "user_query";
pub static RELEVANT_DDLS_VAR_KEY: &str = "relevant_ddls";
pub static FILTER_DDL_TEMPLATE_KEY: &str = "filter_ddls";

// Filters DDLs relevant to the user's query. DDLs can be filtered by one of two
// ways, either through a language model (smart) or cosine similarity (quick).
// Filtering is determined by the  `TableFilterType` value provided.
pub struct DDLFilterRunner<'a> {
    tera: Tera,
    engine: &'a TextToSqlEngine,
    encoder_model: Option<Box<dyn TextEncoderDriver>>,
    instruct_model: Option<Box<dyn TextInstructDriver>>,
}

impl<'a> DDLFilterRunner<'a> {
    pub fn new(engine: &'a TextToSqlEngine) -> Result<Self, SqlgenError> {
        let mut tera = Tera::default();

        tera.add_raw_template(FILTER_DDL_TEMPLATE_KEY, &engine.filter_ddls_template)?;

        let (encoder_model, instruct_model) = match engine.filter_type {
            TableFilterType::Smart => (
                None,
                Some(ModelStore::get_text_instruct_model(&engine.instruct_model)?),
            ),
            TableFilterType::Quick => (
                Some(ModelStore::get_text_encoder_model(&engine.encoder_model)?),
                None,
            ),
        };

        Ok(Self {
            tera,
            engine,
            encoder_model,
            instruct_model,
        })
    }

    // Main entrypoint for runner.
    pub async fn generate(&mut self, user_query: &str) -> Result<Vec<String>, SqlgenError> {
        match self.engine.filter_type {
            TableFilterType::Quick => self.generate_quick(user_query).await,
            TableFilterType::Smart => self.generate_smart(user_query).await,
        }
    }
    fn get_messages(
        tera: &Tera,
        user_query: &str,
        chunk: &[String],
    ) -> Result<Vec<InstructMessage>, SqlgenError> {
        let mut prompt_ctx = Context::new();
        prompt_ctx.insert(USER_QUERY_VAR_KEY, user_query);
        prompt_ctx.insert(RELEVANT_DDLS_VAR_KEY, &chunk.join("\n"));

        let prompt = tera.render(FILTER_DDL_TEMPLATE_KEY, &prompt_ctx)?;

        Ok(vec![InstructMessage {
            role: InstructRole::User,
            message: prompt,
            output_format: Some(ddl_output_format_schema()),
        }])
    }

    // Filter DDLs with a language model.
    async fn generate_smart(&mut self, user_query: &str) -> Result<Vec<String>, SqlgenError> {
        let relevant_ddls = MetadataStore::get_ddls(&self.engine.name)?;
        let model = self.instruct_model.as_deref_mut().ok_or(SqlgenError::Any(
            "generate_smart called without model reference".to_string(),
        ))?;

        let limit = self.engine.column_filter_limit as usize;

        let chunks: Vec<&[String]> = if limit == 0 {
            vec![relevant_ddls.as_slice()]
        } else {
            relevant_ddls.chunks(limit).collect()
        };

        let mut results = Vec::new();

        for chunk in chunks {
            let messages = Self::get_messages(&self.tera, user_query, chunk)?;
            let payload = model.get_assistant_response(messages).await?;

            let response = match from_str::<DDLResponse>(&payload) {
                Ok(v) => Ok(v),
                Err(e) => Err(SqlgenError::GenerateParseError(e.to_string())),
            }?;

            let ddls = match response.error.filter(|s| !s.is_empty()) {
                Some(s) => Err(SqlgenError::GenerateError(s)),
                None => match response.ddls {
                    Some(l) => Ok(l),
                    None => Err(SqlgenError::EmptyResponse),
                },
            }?;

            results.extend(ddls);
        }

        Ok(results)
    }

    // Filter DDLs with cosine similarity.
    async fn generate_quick(&mut self, user_query: &str) -> Result<Vec<String>, SqlgenError> {
        let model = self.encoder_model.as_deref_mut().ok_or(SqlgenError::Any(
            "generate_quick called without model reference".to_string(),
        ))?;
        let encoding = model.encode(user_query).await?;

        MetadataStore::get_similar_ddls(&self.engine.name, encoding, 100)
    }
}

#[cfg(any(test, feature = "pg_test"))]
#[pgrx::pg_schema]
mod tests {

    use crate::{
        pg_test,
        runners::ddl_filter::DDLFilterRunner,
        stores::{metadata_store::MetadataStore, model_store::ModelStore},
        utils::{
            globals::get_runtime,
            test_utils::{create_engine, create_schema},
        },
    };

    #[pg_test]
    fn get_ddl_prompt_output() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_instruct_output = r#"{"ddls": [
            "ddl1", "ddl2", "ddl3", "ddl4", "ddl5",
            "ddl6", "ddl7", "ddl8", "ddl9", "ddl10",
            "ddl11", "ddl12", "ddl13", "ddl14", "ddl15",
            "ddl16", "ddl17", "ddl18", "ddl19", "ddl20"
        ]}"#;

        let engine = create_engine(engine_name, schema_name, "quick", expected_instruct_output);
        let user_query = "Test user query.";
        let runner = DDLFilterRunner::new(&engine).unwrap();
        let ddls = vec![
            "ddl1", "ddl2", "ddl3", "ddl4", "ddl5", "ddl6", "ddl7", "ddl8", "ddl9", "ddl10",
            "ddl11", "ddl12", "ddl13", "ddl14", "ddl15", "ddl16", "ddl17", "ddl18", "ddl19",
            "ddl20",
        ];

        let ddls: Vec<String> = ddls.into_iter().map(|s| s.to_string()).collect();
        let ddls: &[String] = &ddls;
        let messages = DDLFilterRunner::get_messages(&runner.tera, user_query, ddls).unwrap();
        let prompt = messages[0].message.clone();
        let ddl_string = ddls.join("\n");

        assert!(prompt.contains(user_query));
        assert!(prompt.contains(&ddl_string));
    }

    #[pg_test]
    fn test_filter_smart() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_instruct_output = r#"{"ddls": [
            "ddl1", "ddl2", "ddl3", "ddl4", "ddl5",
            "ddl6", "ddl7", "ddl8", "ddl9", "ddl10",
            "ddl11", "ddl12", "ddl13", "ddl14", "ddl15",
            "ddl16", "ddl17", "ddl18", "ddl19", "ddl20"
        ]}"#;
        let engine = create_engine(engine_name, schema_name, "smart", expected_instruct_output);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();
        let user_query = "Test user query.";

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap();
            let mut runner = DDLFilterRunner::new(&engine).unwrap();
            let ddls = runner.generate(user_query).await.unwrap();

            assert_eq!(ddls.len(), 20);
        });
    }

    #[pg_test]
    fn test_filter_smart_error() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_instruct_output = r#"{"error": "test error"}"#;
        let engine = create_engine(engine_name, schema_name, "smart", expected_instruct_output);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();
        let user_query = "Test user query.";

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap();
            let mut runner = DDLFilterRunner::new(&engine).unwrap();
            assert_eq!(
                runner.generate(user_query).await.unwrap_err().to_string(),
                "test error"
            );
        });
    }

    #[pg_test]
    fn test_filter_fast() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_instruct_output = r#"{"ddls": ["ddl1", "ddl2", "ddl3", "ddl4"]}"#;
        let engine = create_engine(engine_name, schema_name, "quick", expected_instruct_output);
        let encoder = ModelStore::get_text_encoder_model(&engine.encoder_model).unwrap();
        let rt = get_runtime();
        let user_query = "Test user query.";

        create_schema(schema_name);

        rt.block_on(async {
            MetadataStore::initialize_metadata(engine_name, schema_name, encoder)
                .await
                .unwrap();
            let mut runner = DDLFilterRunner::new(&engine).unwrap();
            let ddls = runner.generate(user_query).await.unwrap();

            assert_eq!(ddls.len(), 18);
        });
    }
}
