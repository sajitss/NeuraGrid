use async_trait::async_trait;
use super::Capability;

pub struct ObjectTracking;

#[async_trait]
impl Capability for ObjectTracking {
    fn code(&self) -> &'static str {
        "Ot"
    }

    async fn is_supported(&self) -> bool {
        true
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, String> {
        Ok("Object Tracking executed".to_string())
    }
}
