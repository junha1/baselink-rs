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

use super::*;
use crate::port::server::port_thread_local;
pub use intertrait::{cast::CastBox, Caster};
use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

impl<T: ?Sized + Service + ExportService<T>> Serialize for SArc<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let service = self.take();
        let handle = T::export(port_thread_local::get_key(), service);
        handle.serialize(serializer)
    }
}

impl<'de, T: ?Sized + Service + ImportService<T>> Deserialize<'de> for SArc<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, {
        let handle = HandleInstance::deserialize(deserializer)?;
        Ok(SArc::new(T::import(handle)))
    }
}
