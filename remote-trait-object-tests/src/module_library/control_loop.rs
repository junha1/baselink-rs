// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::bootstrap::*;
use super::context::*;
use cbsb::execution::executee;
use cbsb::ipc::{intra, Ipc};
use super::DefaultIpc;
use parking_lot::RwLock;
use remote_trait_object::*;
use std::collections::HashMap;
use std::sync::Arc;

pub fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ctx: &executee::Context<I>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

pub fn send<I: Ipc, T: serde::Serialize>(ctx: &executee::Context<I>, data: &T) {
    ctx.ipc.as_ref().unwrap().send(&serde_cbor::to_vec(data).unwrap());
}

fn create_port(
    port_id: PortId,
    ipc_type: Vec<u8>,
    ipc_config: Vec<u8>,
    dispatcher: Arc<PortDispatcher>,
    instance_key: InstanceKey,
    config_remote_trait_object: &RtoConfig,
) -> Port {
    let ipc_type: String = serde_cbor::from_slice(&ipc_type).unwrap();

    if ipc_type == "DomainSocket" {
        let ipc = DefaultIpc::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(send, recv, port_id, dispatcher, instance_key, config_remote_trait_object)
    } else if ipc_type == "Intra" {
        let ipc = intra::Intra::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(send, recv, port_id, dispatcher, instance_key, config_remote_trait_object)
    } else {
        panic!("Invalid port creation request");
    }
}

pub type DebugFunction = Box<dyn Fn(Vec<u8>) -> Vec<u8>>;
/// initializer will be called after the module configuration is setup.
/// Please initialize your own custom context using it.
pub fn run_control_loop<I: Ipc, H: HandlePreset>(
    args: Vec<String>,
    initializer: Box<dyn Fn() -> ()>,
    debug: Option<DebugFunction>,
) {
    let ctx = executee::start::<I>(args);

    let id_map: IdMap = recv(&ctx);
    let config: Config = recv(&ctx);
    let config_remote_trait_object: RtoConfig = recv(&ctx);
    let _id = config.id.clone();
    let instance_key: InstanceKey = config.key;
    // set instance key also of this main thread.
    set_key(instance_key);
    setup_identifiers(instance_key, &id_map);
    let ports = RwLock::new(PortTable {
        config_remote_trait_object: config_remote_trait_object.clone(),
        map: HashMap::new(),
    });
    global::set(ports);
    super::context::set_module_config(config);
    initializer();
    termination::set(std::sync::atomic::AtomicBool::new(false));

    loop {
        let message: String = recv(&ctx);
        if message == "link" {
            let (port_id, counter_port_id, counter_module_id, ipc_type, ipc_config) = recv(&ctx);
            let dispather = Arc::new(PortDispatcher::new(port_id, 128));
            let mut port_table = global::get().write();

            let old = port_table.map.insert(
                port_id,
                (
                    counter_module_id,
                    counter_port_id,
                    create_port(port_id, ipc_type, ipc_config, dispather, instance_key, &config_remote_trait_object),
                ),
            );
            // we assert before drop old to avoid (hard-to-debug) blocking.
            assert!(old.is_none(), "You must unlink first to link an existing port");
        } else if message == "unlink" {
            let (port_id,) = recv(&ctx);
            let mut port_table = global::get().write();
            port_table.map.remove(&port_id).unwrap();
        } else if message == "terminate" {
            break
        } else if message == "handle_export" {
            // export a default, preset handles for a specific port
            send(&ctx, &H::export());
        } else if message == "handle_import" {
            // import a default, preset handles for a specific port
            let (handles,) = recv(&ctx);
            H::import(handles);
        } else if message == "debug" {
            // temporarily give the execution flow to module, and the module
            // may do whatever it wants but must return a result to report back
            // to host.
            let (args,) = recv(&ctx);
            let result = debug.as_ref().expect("You didn't provide any debug routine")(args);
            send(&ctx, &result);
        } else {
            panic!("Unexpected message: {}", message)
        }
        send(&ctx, &"done".to_owned());
    }
    termination::get().store(true, std::sync::atomic::Ordering::Relaxed);
    super::context::remove_module_config();
    ctx.terminate();
}
