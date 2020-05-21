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
#[macro_use]
mod provider;

use crate::port::Port;
use crate::port::PortId;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RemoteTraitConfig {
    /// Number of inbound call handlers
    pub server_threads: usize,
    /// Maximum outbound call slots
    pub call_slots: usize,
}

/// The entire global context that is enough to make services function.
pub struct PortTable {
    pub config_remote_trait: RemoteTraitConfig,
    /// (Counterparty module's name, Counterarty's port id, Actual port)
    /// TODO: Though it uses HashMap now, we can issue PortIds in a series from 0 to ...
    /// Thus it may be optimized to use plain array later.
    pub map: HashMap<PortId, (String, PortId, Port)>,
}

/// This manages thread-local keys for module instance discrimination
/// in the intra-process setup.
/// This instance key setup will happen always but
/// costly global context resolution will be optionally compiled only with the
/// --features "single_process".
/// Note that you must manually set this key before invoke any call if you created
/// threads during service handling
pub mod single_process_support {
    pub type InstanceKey = u32;
    use std::cell::Cell;
    thread_local!(static INSTANCE_KEY: Cell<InstanceKey> = Cell::new(0));

    pub fn set_key(key: InstanceKey) {
        INSTANCE_KEY.with(|k| {
            assert_eq!(k.get(), 0, "You must set the instance key on your thread");
            k.set(key);
        })
    }

    pub fn get_key() -> InstanceKey {
        INSTANCE_KEY.with(|k| {
            assert_ne!(k.get(), 0, "You must set the instance key on your thread");
            k.get()
        })
    }
}
pub use single_process_support::InstanceKey;

pub mod global {
    use super::*;
    #[cfg(feature ="single_process")]
    use single_process_support as remote_trait;

    context_provider! {RwLock<PortTable>}
    pub fn get() -> &'static Context {
        context_provider_mod::get()
    }

    pub fn set(ctx: Context) {
        context_provider_mod::set(ctx)
    }

    pub fn remove() {
        context_provider_mod::remove()
    }
}

/// Termination flag. If this is set true, drop() from imported service will be ignored.
/// It is module author's job to utilize this.
pub mod termination {
    #[cfg(feature ="single_process")]
    use super::single_process_support as remote_trait;

    context_provider! {std::sync::atomic::AtomicBool}
    pub fn get() -> &'static Context {
        context_provider_mod::get()
    }

    pub fn set(ctx: Context) {
        context_provider_mod::set(ctx)
    }

    pub fn remove() {
        context_provider_mod::remove()
    }
}
