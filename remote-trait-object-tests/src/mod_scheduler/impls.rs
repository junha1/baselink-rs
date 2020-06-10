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

use super::MyContext;
use crate::module_library::prelude::*;
use crate::services::*;
use std::sync::Arc;

#[remote_trait_object_macro::service_impl(Schedule)]
pub struct MySchedule {
    pub handle: HandleInstance,
    pub ctx: Arc<MyContext>,
}

impl Schedule for MySchedule {
    fn get(&self) -> AvailiableMap {
        let mut avail = self.ctx.lock.lock();
        while !*avail {
            self.ctx.cvar.wait(&mut avail);
        }
        *avail = false;
        self.ctx.map.lock().clone()
    }

    fn set(&self, s: AvailiableMap) {
        *self.ctx.map.lock() = s;
        *self.ctx.lock.lock() = true;
        self.ctx.cvar.notify_one();
    }
}
