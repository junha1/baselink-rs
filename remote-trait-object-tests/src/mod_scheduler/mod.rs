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
use std::sync::{Arc, Weak};

#[derive(Debug)]
pub struct MyContext {
    number: usize,
    map: Mutex<AvailiableMap>,
    lock: Mutex<bool>,
    cvar: Condvar,
}

pub struct MyBootstrap {
    ctx: Arc<MyContext>,
}

impl Bootstrap for MyBootstrap {
    fn new(config: &Config) -> Self {
        let (number, threads): (usize, usize) = serde_cbor::from_slice(&config.args).unwrap();
        let map = new_avail_map(number, threads);
        MyBootstrap {
            ctx: Arc::new(MyContext {
                number,
                map: Mutex::new(map),
                lock: Mutex::new(true),
                cvar: Condvar::new(),
            }),
        }
    }

    fn export(&mut self, ports: &mut PortTable) -> Vec<HandleExchange> {
        let mut result = Vec::new();
        for i in 0..self.ctx.number {
            let importer = format!("Module{}", i);

            result.push(HandleExchange {
                exporter: "Schedule".to_owned(),
                importer: importer.clone(),
                handles: vec![service_export!(
                    Schedule,
                    ports.get(&importer).unwrap().get_port(),
                    Arc::new(MySchedule {
                        handle: Default::default(),
                        ctx: self.ctx.clone()
                    })
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(&mut self, _port: Weak<dyn Port>, _exchange: HandleExchange) {
        panic!("Nothing to import!")
    }

    fn debug(&self, _arg: &[u8]) -> Vec<u8> {
        panic!("Nothing to debug!")
    }
}

#[cfg(not(feature = "process"))]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<cbsb::ipc::intra::Intra, MyBootstrap>(args);
}

#[cfg(feature = "process")]
pub fn main_like(args: Vec<String>) {
    run_control_loop::<crate::module_library::DefaultIpc, MyBootstrap>(args);
}
