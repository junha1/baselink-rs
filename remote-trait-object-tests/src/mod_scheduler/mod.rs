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
use parking_lot::{Condvar, Mutex};
use remote_trait_object::*;
use std::sync::Arc;

pub struct MyContext {
    number: usize,
    map: Mutex<AvailiableMap>,
    lock: Mutex<bool>,
    cvar: Condvar,
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
    let (number, threads): (usize, usize) = serde_cbor::from_slice(&config.args).unwrap();
    let map = new_avail_map(number, threads);
    set_context(MyContext {
        number,
        map: Mutex::new(map),
        lock: Mutex::new(true),
        cvar: Condvar::new(),
    });
}

pub struct Preset;

impl HandlePreset for Preset {
    fn export() -> Vec<HandleExchange> {
        let ctx = get_context();
        let mut result = Vec::new();
        for i in 0..ctx.number {
            let importer = format!("Module{}", i);

            result.push(HandleExchange {
                exporter: "Schedule".to_owned(),
                importer: importer.clone(),
                handles: vec![service_export!(
                    Schedule,
                    find_port_id(&importer).unwrap(),
                    Arc::new(MySchedule {
                        handle: Default::default(),
                    })
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(_exchange: HandleExchange) {
        panic!("Nothing to import!")
    }
}

#[cfg(feature = "single_process")]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<cbsb::ipc::intra::Intra, Preset>(args, Box::new(initializer), None);
    remove_context();
    remote_trait_object::global::remove();
}

#[cfg(not(feature = "single_process"))]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<crate::module_library::DefaultIpc, Preset>(args, Box::new(initializer), None);
    remove_context();
    remote_trait_object::global::remove();
}
