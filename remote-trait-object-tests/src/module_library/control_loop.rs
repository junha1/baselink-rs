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

use super::context::*;
use super::DefaultIpc;
use super::PortTable;
use cbsb::execution::executee;
use cbsb::ipc::{intra, Ipc};
use remote_trait_object::*;
use std::sync::Arc;

pub fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ctx: &executee::Context<I>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

pub fn send<I: Ipc, T: serde::Serialize>(ctx: &executee::Context<I>, data: &T) {
    ctx.ipc.as_ref().unwrap().send(&serde_cbor::to_vec(data).unwrap());
}

fn create_port(
    ipc_type: Vec<u8>,
    ipc_config: Vec<u8>,
    dispatcher: Arc<PortDispatcher>,
    config_remote_trait_object: &RtoConfig,
) -> PortInstance {
    let ipc_type: String = serde_cbor::from_slice(&ipc_type).unwrap();

    if ipc_type == "DomainSocket" {
        let ipc = DefaultIpc::new(ipc_config);
        let (send, recv) = ipc.split();
        PortInstance::new(send, recv, dispatcher, config_remote_trait_object)
    } else if ipc_type == "Intra" {
        let ipc = intra::Intra::new(ipc_config);
        let (send, recv) = ipc.split();
        PortInstance::new(send, recv, dispatcher, config_remote_trait_object)
    } else {
        panic!("Invalid port creation request");
    }
}

pub fn run_control_loop<I: Ipc, C: Bootstrap>(args: Vec<String>) {
    let ctx = executee::start::<I>(args);

    let id_map: IdMap = recv(&ctx);
    let config: Config = recv(&ctx);
    let config_remote_trait_object: RtoConfig = recv(&ctx);
    let _id = config.id.clone();
    setup_identifiers(&id_map);
    let mut ports = PortTable::new();
    let mut module_context = C::new(&config);

    loop {
        let message: String = recv(&ctx);
        if message == "link" {
            let (counter_module_id, ipc_type, ipc_config) = recv(&ctx);
            let dispatcher = Arc::new(PortDispatcher::new(128));

            let old = ports
                .insert(counter_module_id, create_port(ipc_type, ipc_config, dispatcher, &config_remote_trait_object));
            // we assert before dropping old to avoid (hard-to-debug) blocking.
            assert!(old.is_none(), "You must unlink first to link an existing port");
        } else if message == "unlink" {
            let (counter_module_id,): (String,) = recv(&ctx);
            ports.remove(&counter_module_id).unwrap();
        } else if message == "terminate" {
            break
        } else if message == "handle_export" {
            // export a default, preset handles for a specific port
            send(&ctx, &module_context.export(&mut ports));
        } else if message == "handle_import" {
            // import a default, preset handles for a specific port
            let (handle,): (HandleExchange,) = recv(&ctx);
            module_context.import(ports.get(&handle.exporter).unwrap().get_port(), handle);
        } else if message == "debug" {
            // temporarily give the execution flow to module, and the module
            // may do whatever it wants but must return a result to report back
            // to host.
            let (args,): (Vec<u8>,) = recv(&ctx);
            let result = module_context.debug(&args);
            send(&ctx, &result);
        } else {
            panic!("Unexpected message: {}", message)
        }
        send(&ctx, &"done".to_owned());
    }

    for port in ports.values() {
        port.ready_to_terminate();
    }
    drop(module_context);
    drop(ports);

    ctx.terminate();
}
