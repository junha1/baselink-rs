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

use remote_trait::{context_provider, InstanceKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// kind of this module. Per-binary
    pub kind: String,
    /// id of this instance of module. Per-instance, Per-appdescriptor
    pub id: String,
    /// key of this instance of module. Per-instance, Per-execution, Per-node
    pub key: InstanceKey,
    /// Arguments given to this module.
    pub args: Vec<u8>,
}

context_provider! {Config}
pub fn get_module_config() -> &'static Config {
    context_provider_mod::get()
}

pub(crate) fn set_module_config(ctx: Config) {
    context_provider_mod::set(ctx)
}

pub(crate) fn remove_module_config() {
    context_provider_mod::remove()
}
