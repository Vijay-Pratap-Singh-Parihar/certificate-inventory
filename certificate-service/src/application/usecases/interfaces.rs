use async_trait::async_trait;
use crate::application::errors::application_error::ApplicationError;

#[async_trait]
pub trait AbstractUseCase<Output, Input> {
    async fn execute(&self, input: Option<Input>) -> Result<Output, ApplicationError>;
}
