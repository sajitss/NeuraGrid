use async_trait::async_trait;
use super::Capability;

pub struct VideoSearch;

#[async_trait]
impl Capability for VideoSearch {
    fn code(&self) -> &'static str {
        "Vs"
    }

    async fn is_supported(&self) -> bool {
        true
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, String> {
        Ok("Video Search executed".to_string())
    }
}
