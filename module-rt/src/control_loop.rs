use crate::core::{Core, CoreMessage};
use cbsb::execution::executee;
use cbsb::ipc::Ipc;
use remote_trait_object::*;
use std::sync::Arc;

pub fn control_loop<I: Ipc>(mut core: crate::core::Core<I>) {
    loop {
        let msg = core.get_message();
        match msg {
            CoreMessage::CreatePort(x) => panic!(),
            CoreMessage::Debug(_) => panic!(),
            CoreMessage::Terminate => panic!(),
        }
    }
}
