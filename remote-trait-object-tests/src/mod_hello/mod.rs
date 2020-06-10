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
use remote_trait_object::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

pub struct MyContext {
    number: usize,
    factories: RwLock<HashMap<String, Arc<dyn HelloFactory>>>,
    config: Config,
}

pub struct MyBootstrap {
    ctx: Arc<MyContext>,
}

impl Bootstrap for MyBootstrap {
    fn new(config: &Config) -> Self {
        let number = serde_cbor::from_slice(&config.args).unwrap();
        let mut factories = HashMap::new();
        factories.insert(
            config.id.clone(),
            Arc::new(Factory {
                handle: Default::default(),
            }) as Arc<dyn HelloFactory>,
        );

        MyBootstrap {
            ctx: Arc::new(MyContext {
                number,
                factories: RwLock::new(factories),
                config: config.clone(),
            }),
        }
    }

    fn export(&mut self, ports: &mut PortTable) -> Vec<HandleExchange> {
        let mut result = Vec::new();
        for i in 0..self.ctx.number {
            let exporter = self.ctx.config.id.clone();
            let importer = format!("Module{}", i);
            if exporter == importer {
                continue
            }

            let x = ports.get(&importer).unwrap().get_port();
            assert_ne!(x.strong_count(), 0);

            result.push(HandleExchange {
                exporter,
                importer: importer.clone(),
                handles: vec![service_export!(
                    HelloFactory,
                    ports.get(&importer).unwrap().get_port(),
                    Arc::new(Factory {
                        handle: Default::default(),
                    })
                )],
                argument: Vec::new(),
            })
        }
        result
    }

    fn import(&mut self, port: Weak<dyn Port>, mut exchange: HandleExchange) {
        assert_eq!(exchange.importer, self.ctx.config.id, "Invalid import request");
        let mut guard = self.ctx.factories.write();
        assert_eq!(exchange.handles.len(), 1);
        let h = service_import!(HelloFactory, port, exchange.handles.pop().unwrap());
        guard.insert(exchange.exporter, h);
    }

    fn debug(&self, _arg: &[u8]) -> Vec<u8> {
        let guard = self.ctx.factories.read();

        for n in 0..self.ctx.number {
            let factory = guard.get(&format!("Module{}", n)).unwrap();
            for i in 0..10 {
                let robot = factory.create(&format!("Robot{}", i)).unwrap();
                assert_eq!(robot.hello(10 - i), format!("Robot{}{}", i, 10 - i));
            }
        }
        Vec::new()
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
