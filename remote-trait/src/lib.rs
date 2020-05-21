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

pub mod ipc;
mod context;
mod port;
pub mod queue;
mod service;
pub mod statistics;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[macro_use]
extern crate intertrait;

pub use context::{
    global, single_process_support::get_key, single_process_support::set_key, termination, InstanceKey, PortTable,
    RemoteTraitConfig,
};
pub use port::{PacketHeader, Port, PortId};
pub use service::id::{setup_identifiers, IdMap};
pub use service::SArc;
pub use service::{
    dispatch::PortDispatcher, dispatch::ServiceDispatcher, HandleInstance, MethodId, Service, ServiceObjectId, TraitId,
};

/// You should not import this! This is for the auto-generated code
pub mod env {
    pub use crate::context::global;
    pub use crate::port::{PacketHeader, Port, PortId};
    pub use crate::service::dispatch::ServiceDispatcher;
    pub use crate::service::id::{MID_REG, TID_REG};
    pub use crate::service::service_context;
    pub use crate::service::{DispatchService, ExportService, IdOfService, ImportService, SArc};
    pub use crate::service::{HandleInstance, MethodId, MethodIdAtomic, Service, TraitId, TraitIdAtomic, ID_ORDERING};
}

/// You should not import this! This is for the auto-generated code
pub mod env_mock {
    pub use crate::context::global;
    pub use crate::port::{PacketHeader, Port, PortId};
    pub use crate::service::dispatch::ServiceDispatcher;
    pub use crate::service::id::{MID_REG, TID_REG};
    pub use crate::service::{DispatchService, ExportService, IdOfService, ImportService, SArc};
    pub use crate::service::{HandleInstance, MethodId, MethodIdAtomic, Service, TraitId, TraitIdAtomic, ID_ORDERING};
}
