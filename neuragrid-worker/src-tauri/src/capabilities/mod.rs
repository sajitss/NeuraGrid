use async_trait::async_trait;

pub mod pd;
pub mod pr;
pub mod ot;
pub mod vd;
pub mod lp;
pub mod vs;

#[async_trait]
pub trait Capability: Send + Sync {
    fn code(&self) -> &'static str;
    async fn is_supported(&self) -> bool;
    async fn execute(&self, args: Vec<String>) -> Result<String, String>;
}
