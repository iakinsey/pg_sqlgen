use crate::types::traits::driver::TextInstructDriver;

pub struct SQLGenerationRunner {
    template: String,
    model: Box<dyn TextInstructDriver>,
}

impl SQLGenerationRunner {
    pub fn new(model: Box<dyn TextInstructDriver>, template: &str) -> Self {
        Self {
            template: template.to_string(),
            model,
        }
    }

    pub async fn generate_query(&mut self, model: Box<dyn TextInstructDriver>, prompt: &str) {
        unimplemented!()
    }
}
