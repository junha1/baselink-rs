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
mod port;
pub mod queue;
mod service;
pub mod statistics;
#[cfg(test)]
mod tests;
#[cfg(test)]
#[macro_use]
extern crate intertrait;

/// You (module implementor) should not refer this!
pub mod macro_env {
    use super::*;
    pub use port::{PacketHeader, Port};
    pub use service::id::{MethodIdAtomic, TraitIdAtomic, ID_ORDERING, MID_REG, TID_REG};
    pub use service::{dispatch::ServiceDispatcher, DispatchService, ExportService, ImportService};
    pub use service::{HandleInstance, HandleToExchange, IdOfService, MethodId, Service, TraitId};
}

pub use port::{BasicPort, PacketHeader, Port, PortInstance};
pub use service::id::*;
pub use service::SArc;
pub use service::{
    dispatch::PortDispatcher, dispatch::ServiceDispatcher, ExportService, HandleInstance, HandleToExchange,
    ImportService, MethodId, Service, ServiceObjectId, TraitId,
};

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone)]
pub struct RtoConfig {
    /// Number of inbound call handlers
    pub server_threads: usize,
    /// Maximum outbound call slots
    pub call_slots: usize,
}
