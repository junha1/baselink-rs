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

mod impls;

use crate::module_library::*;
use crate::services::*;
use impls::*;
use parking_lot::RwLock;
use remote_trait::*;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MyContext {
    number: usize,
    factories: RwLock<HashMap<String, Arc<dyn HelloFactory>>>,
}

context_provider! {MyContext}
pub fn get_context() -> &'static MyContext {
    context_provider_mod::get()
}
pub fn set_context(ctx: MyContext) {
    context_provider_mod::set(ctx)
}
pub fn remove_context() {
    context_provider_mod::remove()
}

pub fn initializer() {
    let config = get_module_config();
    let number = serde_cbor::from_slice(&config.args).unwrap();
    let mut factories = HashMap::new();
    factories.insert(
        config.id.clone(),
        Arc::new(Factory {
            handle: Default::default(),
        }) as Arc<dyn HelloFactory>,
    );
    set_context(MyContext {
        number,
        factories: RwLock::new(factories),
    });
}

pub struct Preset;

impl HandlePreset for Preset {
    fn export() -> Vec<HandleExchange> {
        let ctx = get_context();
        let mut result = Vec::new();
        for i in 0..ctx.number {
            let exporter = get_module_config().id.clone();
            let importer = format!("Module{}", i);
            if exporter == importer {
                continue
            }

            result.push(HandleExchange {
                exporter,
                importer: importer.clone(),
                handles: vec![service_export!(
                    HelloFactory,
                    find_port_id(&importer).unwrap(),
                    Arc::new(Factory {
                        handle: Default::default(),
                    })
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(mut exchange: HandleExchange) {
        let ctx = get_context();
        assert_eq!(exchange.importer, get_module_config().id, "Invalid import request");
        let mut guard = ctx.factories.write();
        assert_eq!(exchange.handles.len(), 1);
        let h = service_import!(HelloFactory, exchange.handles.pop().unwrap());
        guard.insert(exchange.exporter, h);
    }
}

pub fn initiate(_arg: Vec<u8>) -> Vec<u8> {
    let ctx = get_context();
    let guard = ctx.factories.read();

    for n in 0..ctx.number {
        let factory = guard.get(&format!("Module{}", n)).unwrap();
        for i in 0..10 {
            let robot = factory.create(&format!("Robot{}", i)).unwrap();
            assert_eq!(robot.hello(10 - i), format!("Robot{}{}", i, 10 - i));
        }
    }
    Vec::new()
}

#[cfg(feature = "single_process")]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<cbsb::ipc::intra::Intra, Preset>(args, Box::new(initializer), Some(Box::new(initiate)));
    remove_context();
    remote_trait::global::remove();
}

#[cfg(not(feature = "single_process"))]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<cbsb::ipc::servo_channel::ServoChannel, Preset>(
        args,
        Box::new(initializer),
        Some(Box::new(initiate)),
    );
    remove_context();
    remote_trait::global::remove();
}
