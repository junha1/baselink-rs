use crate as remote_trait_object;
use crate as rto;

use rto::SArc;
use rto::Service;
use std::sync::Arc;

#[remote_trait_object_macro::service_adv(serde_cbor)]
pub trait TestService: remote_trait_object::Service {
    fn fn1(&self, a1: String, a2: &str, a3: &[u8]) -> SArc<dyn TestService>;

    fn fn2(&self, a2: &u8) -> String;

    fn fn3(&self) -> String;
}

#[remote_trait_object_macro::service_impl_adv(TestService)]
pub struct TestImpl {
    pub handle: remote_trait_object::HandleInstance,
    pub name: String,
}

#[cast_to([sync])]
impl TestService for TestImpl {
    fn fn1(&self, a1: String, a2: &str, a3: &[u8]) -> SArc<dyn TestService> {
        SArc::new(Arc::new(TestImpl {
            handle: Default::default(),
            name: format!("{}{}{}", a1, a2, a3.len()),
        }))
    }

    fn fn2(&self, a2: &u8) -> String {
        format!("{}", a2)
    }

    fn fn3(&self) -> String {
        self.name.clone()
    }
}
