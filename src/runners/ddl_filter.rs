use crate::{
    stores::{metadata_store::MetadataStore, model_store::ModelStore},
    types::{
        errors::SqlgenError,
        structs::{
            engine::{TableFilterType, TextToSqlEngine},
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
};
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
        }])
    }

    // Filter DDLs with a language model.
    async fn generate_smart(&mut self, user_query: &str) -> Result<Vec<String>, SqlgenError> {
        let relevant_ddls = MetadataStore::get_ddls(&self.engine.name)?;
        let model = self.instruct_model.as_deref_mut().ok_or(SqlgenError::Any(
            "generate_smart called without model reference".to_string(),
        ))?;

        let limit = self.engine.ddl_prompt_limit as usize;
        let chunks: Vec<&[String]> = if limit <= 0 {
            vec![relevant_ddls.as_slice()]
        } else {
            relevant_ddls.chunks(limit).collect()
        };

        let mut results = Vec::new();

        for chunk in chunks {
            let messages = Self::get_messages(&self.tera, user_query, chunk)?;
            let response = model.get_assistant_response(messages).await?;

            results.extend(
                response
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| line.to_string()),
            );
        }

        Ok(results)
    }

    // Filter DDLs with cosine similarity.
    async fn generate_quick(&mut self, user_query: &str) -> Result<Vec<String>, SqlgenError> {
        let model = self.encoder_model.as_deref_mut().ok_or(SqlgenError::Any(
            "generate_quick called without model reference".to_string(),
        ))?;
        let encoding = model.encode(user_query).await?;

        Ok(MetadataStore::get_similar_ddls(
            &self.engine.name,
            encoding,
            100,
        )?)
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
        let expected_instruct_output = "ddl1\nddl2\nddl3\nddl4";
        let engine = create_engine(engine_name, schema_name, "quick", expected_instruct_output);
        let user_query = "Test user query.";
        let runner = DDLFilterRunner::new(&engine).unwrap();
        let ddls = vec!["ddl1", "ddl2", "ddl3"];
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
        let expected_instruct_output = "ddl1\nddl2\nddl3\nddl4";
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

            assert_eq!(ddls.len(), 4);
        });
    }

    #[pg_test]
    fn test_filter_fast() {
        let schema_name = "test_example";
        let engine_name = "test_engine";
        let expected_instruct_output = "ddl1\nddl2\nddl3\nddl4";
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
