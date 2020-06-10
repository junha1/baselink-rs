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
use rand::{rngs::StdRng, Rng};
use remote_trait_object::*;
use std::collections::HashMap;
use std::sync::{Arc, Weak};
use std::thread;

#[derive(Debug)]
pub struct MyContext {
    config: Config,

    /// total number of relayers
    number: usize,
    /// My index
    index: usize,
    schedule: RwLock<Option<Arc<dyn Schedule>>>,
    factories: RwLock<HashMap<String, Arc<dyn RelayerFactory>>>,
    answers: RwLock<HashMap<String, (Vec<String>, String)>>,
}

pub struct MyBootstrap {
    ctx: Arc<MyContext>,
}

impl Bootstrap for MyBootstrap {
    fn new(config: &Config) -> Self {
        let (number, index) = serde_cbor::from_slice(&config.args).unwrap();
        let factories = HashMap::new();

        let ctx = Arc::new(MyContext {
            config: config.clone(),
            number,
            index,
            schedule: Default::default(),
            factories: RwLock::new(factories),
            answers: Default::default(),
        });
        ctx.factories.write().insert(
            config.id.clone(),
            Arc::new(OrdinaryFactory {
                handle: Default::default(),
                ctx: ctx.clone(), // FIXME
            }) as Arc<dyn RelayerFactory>,
        );
        MyBootstrap {
            ctx,
        }
    }

    fn export(&mut self, ports: &mut PortTable) -> Vec<HandleExchange> {
        let number = self.ctx.number;
        let mut exchanges = Vec::<HandleExchange>::new();

        for i in 0..number {
            let name = format!("Module{}", i);
            if name == self.ctx.config.id {
                // myself
                continue
            }
            let id = service_export!(
                RelayerFactory,
                ports.get(&format!("Module{}", i)).unwrap().get_port(),
                Arc::new(OrdinaryFactory {
                    handle: Default::default(),
                    ctx: self.ctx.clone()
                })
            );
            exchanges.push(HandleExchange {
                exporter: self.ctx.config.id.clone(),
                importer: name,
                handles: vec![id],
                argument: Vec::new(),
            })
        }
        exchanges
    }

    fn import(&mut self, port: Weak<dyn Port>, mut exchange: HandleExchange) {
        assert_eq!(exchange.importer, self.ctx.config.id, "Invalid import request");
        if exchange.exporter == "Schedule" {
            assert_eq!(exchange.handles.len(), 1);
            self.ctx.schedule.write().replace(service_import!(Schedule, port, exchange.handles.pop().unwrap()));
        } else {
            let mut guard = self.ctx.factories.write();
            assert_eq!(exchange.handles.len(), 1);
            let h = service_import!(RelayerFactory, port, exchange.handles.pop().unwrap());
            guard.insert(exchange.exporter, h);
        }
    }

    fn debug(&self, _arg: &[u8]) -> Vec<u8> {
        let my_factory = OrdinaryFactory {
            handle: Default::default(),
            ctx: self.ctx.clone(),
        };

        let mut rng: StdRng = rand::SeedableRng::from_entropy();

        let iteration = 32;
        let parllel: usize = 2;
        let my_index = self.ctx.index;
        let number = self.ctx.number;

        for _ in 0..iteration {
            let mut used_map_list = HashMap::new();
            let mut paths = HashMap::new();
            for i in 0..parllel {
                let mut used_map = new_avail_map(self.ctx.number, 0);
                let avail = self.ctx.schedule.read().as_ref().unwrap().get();
                // RelayerFactory::ask_path will require one thread always
                let mut at_least_1_for_all = true;
                for j in 0..number {
                    if j == my_index {
                        continue
                    }
                    if avail[my_index][j] < 1 {
                        at_least_1_for_all = false;
                        break
                    }
                }
                if !at_least_1_for_all {
                    self.ctx.schedule.read().as_ref().unwrap().set(avail);
                    continue
                }
                let mut avail = avail;
                for j in 0..number {
                    if j == my_index {
                        continue
                    }
                    avail[my_index][j] -= 1;
                    used_map[my_index][j] += 1;
                }

                // path generation
                let mut path = Vec::new();
                let mut last = my_index;
                for _ in 0..30 {
                    let mut suc = false;
                    for _ in 0..5 {
                        let next = rng.gen_range(0, number);
                        // no consumption of thread here (See how we set the factory handle for itself)
                        if next == last {
                            path.push(format!("Module{}", next));
                            suc = true;
                            break
                        } else if avail[next][last] > 0 {
                            path.push(format!("Module{}", next));
                            avail[next][last] -= 1;
                            used_map[next][last] += 1;
                            last = next;
                            suc = true;
                            break
                        }
                    }
                    if !suc {
                        break
                    }
                }
                path.insert(0, self.ctx.config.id.clone());
                let key = format!("Key{}", i);
                paths.insert(key.clone(), path);
                used_map_list.insert(key.clone(), used_map);

                self.ctx.schedule.read().as_ref().unwrap().set(avail);
            }

            {
                let mut guard_answers = self.ctx.answers.write();
                guard_answers.clear();
                for (key, path) in paths.drain() {
                    guard_answers.insert(key, (path, format!("{}", rng.gen_range(0, 10000))));
                }
            }

            let guard_answers = self.ctx.answers.read();
            let guard_factory = self.ctx.factories.read();
            let mut runners = Vec::new();
            for (key, (path, answer)) in &*guard_answers {
                if path.len() < 2 {
                    continue
                }
                if let Answer::Next(next) = my_factory.ask_path(key.clone(), 0) {
                    let machine =
                        guard_factory.get(&next).unwrap().create(key.clone(), 0, self.ctx.config.id.clone()).unwrap();
                    runners.push((key.clone(), thread::spawn(move || machine.run()), answer.clone()));
                } else {
                    panic!("Test illformed")
                }
            }

            while let Some((key, guess, answer)) = runners.pop() {
                assert_eq!(guess.join().unwrap(), answer);
                let mut avail = self.ctx.schedule.read().as_ref().unwrap().get();

                for (avail_sub_list, used_sub_list) in
                    avail.iter_mut().zip(used_map_list.remove(&key).unwrap().into_iter())
                {
                    for (avail_entry, used_entry) in avail_sub_list.iter_mut().zip(used_sub_list.iter()) {
                        *avail_entry += *used_entry;
                    }
                }
                self.ctx.schedule.read().as_ref().unwrap().set(avail.clone());
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
