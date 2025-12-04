use crate::capabilities::{Capability, vs, pr};

pub fn get_capabilities() -> Vec<Box<dyn Capability>> {
    vec![
        Box::new(vs::VideoSearch),
        Box::new(pr::PersonReId),
    ]
}
