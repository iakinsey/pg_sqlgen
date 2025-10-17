use crate::{
    stores::model_store::ModelStore,
    types::{
        errors::StoreError,
        structs::{
            engine::{TableFilterType, TextToSqlEngine},
            instruct_message::{InstructMessage, InstructRole},
        },
        traits::driver::{TextEncoderDriver, TextInstructDriver},
    },
};
use tera::{Context, Tera};

pub struct DDLFilterRunner {
    tera: Tera,
    engine: TextToSqlEngine,
    encoder_model: Option<Box<dyn TextEncoderDriver>>,
    instruct_model: Option<Box<dyn TextInstructDriver>>,
}

/*
    Example filter template:

    The user has the following query:

    {user_query}

    Filter this list, only select elements that you believe are relevant to the users query.
    Respond by only returning the following list and nothing else:

    {relevant_ddls}
*/
pub static USER_QUERY_VAR_KEY: &str = "user_query";
pub static RELEVANT_DDLS_VAR_KEY: &str = "relevant_ddls";
pub static FILTER_DDL_TEMPLATE_KEY: &str = "filter_ddls";

// TODO allow for segmenting multiple queries
impl DDLFilterRunner {
    pub fn new(engine: TextToSqlEngine) -> Result<Self, StoreError> {
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

    pub async fn generate(
        &mut self,
        user_query: &str,
        relevant_ddls: Vec<&str>,
    ) -> Result<Vec<String>, StoreError> {
        match self.engine.filter_type {
            TableFilterType::Quick => self.generate_quick(user_query, relevant_ddls).await,
            TableFilterType::Smart => self.generate_smart(user_query, relevant_ddls).await,
        }
    }

    async fn generate_smart(
        &mut self,
        user_query: &str,
        relevant_ddls: Vec<&str>,
    ) -> Result<Vec<String>, StoreError> {
        let mut prompt_ctx = Context::new();
        let relevant_ddls = relevant_ddls.join("\n");

        prompt_ctx.insert(USER_QUERY_VAR_KEY, user_query);
        prompt_ctx.insert(RELEVANT_DDLS_VAR_KEY, &relevant_ddls);

        let prompt = self.tera.render(FILTER_DDL_TEMPLATE_KEY, &prompt_ctx)?;
        let model = self.instruct_model.as_deref_mut().ok_or(StoreError::Any(
            "generate_smart called without model reference".to_string(),
        ))?;

        let messages = vec![InstructMessage {
            role: InstructRole::User,
            message: prompt,
        }];

        Ok(model
            .get_assistant_response(messages)
            .await?
            .lines()
            .map(|line| line.trim().to_string())
            .collect())
    }

    async fn generate_quick(
        &mut self,
        user_query: &str,
        relevant_ddls: Vec<&str>,
    ) -> Result<Vec<String>, StoreError> {
        unimplemented!()
    }
}
