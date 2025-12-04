use async_trait::async_trait;
use super::Capability;

pub struct VehicleDetection;

#[async_trait]
impl Capability for VehicleDetection {
    fn code(&self) -> &'static str {
        "Vd"
    }

    async fn is_supported(&self) -> bool {
        true
    }

    async fn execute(&self, _args: Vec<String>) -> Result<String, String> {
        Ok("Vehicle Detection executed".to_string())
    }
}
