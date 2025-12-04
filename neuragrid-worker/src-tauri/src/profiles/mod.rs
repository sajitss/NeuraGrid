use super::capabilities::Capability;

pub mod standard;
pub mod edge_jetson;
pub mod server_gpu;

pub fn get_active_profile() -> Vec<Box<dyn Capability>> {
    // TODO: Implement logic to select profile based on config or hardware
    standard::get_capabilities()
}
