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

use super::get_context;
use crate::services::*;
use std::sync::Arc;
use test_module_library::prelude::*;

#[remote_trait_macro::service_impl(RelayerFactory)]
pub struct OrdinaryFactory {
    pub handle: HandleInstance,
}

impl RelayerFactory for OrdinaryFactory {
    fn create(&self, key: String, current: usize, destination: String) -> SArc<dyn RelayerMachine> {
        SArc::new(Arc::new(OrdinaryMachine {
            handle: Default::default(),
            key,
            current: current + 1,
            destination,
        }))
    }

    /// Returns name of the next module to visit
    fn ask_path(&self, key: String, current: usize) -> Answer {
        let guard_answers = get_context().answers.read();
        let entry = guard_answers.get(&key).unwrap();
        if current == entry.0.len() - 1 {
            Answer::End(entry.1.clone())
        } else {
            Answer::Next(entry.0[current + 1].clone())
        }
    }
}

#[remote_trait_macro::service_impl(RelayerMachine)]
pub struct OrdinaryMachine {
    pub handle: HandleInstance,
    pub key: String,
    pub current: usize,
    pub destination: String,
}

impl RelayerMachine for OrdinaryMachine {
    fn run(&self) -> String {
        let guard_factory = get_context().factories.read();
        match guard_factory.get(&self.destination).unwrap().ask_path(self.key.clone(), self.current) {
            Answer::Next(x) => guard_factory
                .get(&x)
                .unwrap()
                .create(self.key.clone(), self.current, self.destination.clone())
                .unwrap()
                .run(),
            Answer::End(x) => x,
        }
    }
}
