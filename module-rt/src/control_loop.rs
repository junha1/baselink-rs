use crate::core::{Core, CoreMessage};
use crate::port::Port;
use cbsb::execution::executee;
use cbsb::ipc::Ipc;
use parking_lot::{Mutex, RwLock};
use remote_trait_object::*;
use std::collections::HashMap;
use std::sync::Arc;
use std::thread;

pub fn control_loop<I: Ipc + 'static>(mut core: crate::core::Core<I>) {
    let ports: Arc<RwLock<HashMap<PortId, Mutex<Port<I>>>>> = Default::default();
    let mut port_id_count = 1;
    let mut job_list = Vec::new();

    loop {
        let msg = core.get_message();
        match msg {
            CoreMessage::CreatePort(a1, a2) => {
                let ports = ports.clone();
                let export_pool = core.get_export_pool();
                job_list.push(thread::spawn(move || {
                    let port = Port::<I>::new(port_id_count, a1, a2, export_pool, unimplemented!());
                    ports.write().insert(port_id_count, Mutex::new(port));
                }));

                port_id_count += 1;
            }
            CoreMessage::Debug(_) => panic!(),
            CoreMessage::Terminate => panic!(),
        }
    }

    for x in job_list {
        x.join().unwrap();
    }
}
