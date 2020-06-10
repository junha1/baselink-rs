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
use crate::module_library::prelude::*;
use crate::services::*;

#[remote_trait_object_macro::service_impl(Schedule)]
pub struct MySchedule {
    pub handle: HandleInstance,
}

impl Schedule for MySchedule {
    fn get(&self) -> AvailiableMap {
        let mut avail = get_context().lock.lock();
        while !*avail {
            get_context().cvar.wait(&mut avail);
        }
        *avail = false;
        get_context().map.lock().clone()
    }

    fn set(&self, s: AvailiableMap) {
        *get_context().map.lock() = s;
        *get_context().lock.lock() = true;
        get_context().cvar.notify_one();
    }
}
