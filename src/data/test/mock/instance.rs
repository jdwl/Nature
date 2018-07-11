use data::instance::Instance;
use data::InstanceServiceTrait;
use std::ops::Deref;
use super::*;

lazy_static! {
    pub static ref DATA_INSTANCE_LOCK: Mutex<u8> = Mutex::new(1);
    pub static ref DATA_INSTANCE_RESULT: Mutex<Result<u128>> = Mutex::new(Ok(0));
}

pub struct MockInstanceTrait;

impl InstanceServiceTrait for MockInstanceTrait {
    fn verify(_instance: &mut Instance) -> Result<u128> {
        DATA_INSTANCE_RESULT.lock().unwrap().deref().clone()
    }
}

