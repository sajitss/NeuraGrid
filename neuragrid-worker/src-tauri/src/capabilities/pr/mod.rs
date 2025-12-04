use async_trait::async_trait;
use super::Capability;

pub struct PersonReId;

#[async_trait]
impl Capability for PersonReId {
    fn code(&self) -> &'static str {
        "Pr"
    }

    async fn is_supported(&self) -> bool {
        true
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, String> {
        Ok("Person Re-ID executed".to_string())
    }
}
