use crate::capabilities::{Capability, pd, ot};

pub fn get_capabilities() -> Vec<Box<dyn Capability>> {
    vec![
        Box::new(pd::PeopleDetection),
        Box::new(ot::ObjectTracking),
    ]
}
