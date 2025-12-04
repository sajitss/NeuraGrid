use crate::capabilities::{Capability, pd, pr, ot, vd, lp, vs};

pub fn get_capabilities() -> Vec<Box<dyn Capability>> {
    vec![
        Box::new(pd::PeopleDetection),
        Box::new(pr::PersonReId),
        Box::new(ot::ObjectTracking),
        Box::new(vd::VehicleDetection),
        Box::new(lp::LicensePlateRecognition),
        Box::new(vs::VideoSearch),
    ]
}
