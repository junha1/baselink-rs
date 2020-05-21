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

pub use crate::module_library::prelude::*;

#[remote_trait_macro::service]
pub trait HelloFactory: remote_trait::Service {
    fn create(&self, name: &str) -> SArc<dyn HelloRobot>;
}

#[remote_trait_macro::service]
pub trait HelloRobot: remote_trait::Service {
    fn hello(&self, flag: i32) -> String;
}

#[derive(PartialEq, serde::Serialize, serde::Deserialize, Debug)]
pub enum Answer {
    Next(String),
    End(String),
}

/// module index -> (caller module index -> availiable handlers)
pub type AvailiableMap = Vec<Vec<usize>>;

pub fn new_avail_map(size: usize, value: usize) -> AvailiableMap {
    let mut result = vec![Vec::new(); size];
    for (i, x) in result.iter_mut().enumerate() {
        for j in 0..size {
            if i == j {
                x.push(0);
            } else {
                x.push(value);
            }
        }
    }
    result
}

#[remote_trait_macro::service]
pub trait Schedule: remote_trait::Service {
    /// Get the schedule. It is then locked.
    fn get(&self) -> AvailiableMap;

    /// Set the schedule. It is then unlocked.
    fn set(&self, s: AvailiableMap);
}

#[remote_trait_macro::service]
pub trait RelayerFactory: remote_trait::Service {
    /// Make an invitation for a single visit toward itself
    fn create(&self, key: String, current: usize, destination: String) -> SArc<dyn RelayerMachine>;

    /// Returns name of the next module to visit
    fn ask_path(&self, key: String, current: usize) -> Answer;
}

#[remote_trait_macro::service]
pub trait RelayerMachine: remote_trait::Service {
    /// Recursively traverse all the path and query the answer for the destination
    fn run(&self) -> String;
}
