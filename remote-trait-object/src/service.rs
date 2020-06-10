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

pub mod call;
pub mod dispatch;
pub mod id;
pub mod serde_support;
pub mod table;

use super::port::PortId;
pub use dispatch::PortDispatcher;
use serde::{Deserialize, Serialize};
pub use std::sync::Arc;

pub type MethodId = u32;
pub type TraitId = u16;
pub type InstanceId = u16;

pub const ID_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;
pub type MethodIdAtomic = std::sync::atomic::AtomicU32;
pub type TraitIdAtomic = std::sync::atomic::AtomicU16;

// We avoid using additional space with Option<>, by these.
pub const UNDECIDED_INDEX: InstanceId = std::u16::MAX;
pub const UNDECIDED_PORT: PortId = std::u16::MAX;

/// This struct represents an index to a service object in port server's registry
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ServiceObjectId {
    pub(crate) index: InstanceId,
}

/// This struct is stored in both service object and call stub.
/// Actually, both use only part of the fields respectively,
/// though the other fields will be still set same as
/// the other side's HandleInstance.
/// For debugging and simplicity, we won't split this for now.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct HandleInstance {
    pub(crate) id: ServiceObjectId,
    // That of exporter's.
    pub(crate) port_id_exporter: PortId,
    // That of importer's.
    pub(crate) port_id_importer: PortId,
}

impl Default for HandleInstance {
    fn default() -> Self {
        HandleInstance {
            id: ServiceObjectId {
                index: UNDECIDED_INDEX,
            },
            port_id_exporter: UNDECIDED_PORT,
            port_id_importer: UNDECIDED_PORT,
        }
    }
}

impl HandleInstance {
    /// This clone is allowed only within this crate
    pub(crate) fn careful_clone(&self) -> Self {
        HandleInstance {
            id: self.id,
            port_id_exporter: self.port_id_exporter,
            port_id_importer: self.port_id_importer,
        }
    }

    /// You (module implementor) should not call this!
    pub fn for_dispatcher_get_port_id(&self) -> PortId {
        self.port_id_exporter
    }
}

/// All service trait must has this as a supertrait.
pub trait Service: dispatch::ServiceDispatcher + std::fmt::Debug + intertrait::CastFromSync + Send + Sync {
    fn get_handle(&self) -> &HandleInstance;
    fn get_handle_mut(&mut self) -> &mut HandleInstance;
    fn get_trait_id(&self) -> TraitId;
}

pub struct SArc<T: ?Sized + Service> {
    value: std::cell::Cell<Option<Arc<T>>>,
}

impl<T: ?Sized + Service> SArc<T> {
    pub fn new(value: Arc<T>) -> Self {
        SArc {
            value: std::cell::Cell::new(Some(value)),
        }
    }

    pub(crate) fn take(&self) -> Arc<T> {
        self.value.take().unwrap()
    }

    pub fn unwrap(self) -> Arc<T> {
        self.value.take().unwrap()
    }
}

// These four traits are very special: they are associated with a specific trait.
// However use (module author) will never use these, but the generated code will.
pub trait ImportService<T: ?Sized + Service> {
    fn import(handle: HandleInstance) -> Arc<T>;
}

pub trait ExportService<T: ?Sized + Service> {
    fn export(port_id: PortId, object: Arc<T>) -> HandleInstance;
}

pub trait DispatchService<T: ?Sized + Service> {
    fn dispatch(object: &T, method: MethodId, arguments: &[u8], return_buffer: std::io::Cursor<&mut Vec<u8>>);
}

pub trait IdOfService<T: ?Sized + Service> {
    fn id() -> TraitId;
}

#[macro_export]
macro_rules! service_export {
    ($service_trait: path, $port_id: expr, $arg: expr) => {
        <dyn $service_trait as remote_trait_object::env::ExportService<dyn $service_trait>>::export($port_id, $arg)
    };
}

#[macro_export]
macro_rules! service_import {
    ($service_trait: path, $arg: expr) => {
        <dyn $service_trait as remote_trait_object::env::ImportService<dyn $service_trait>>::import($arg)
    };
}

#[macro_export]
macro_rules! service_dispatch {
    ($service_trait: path, $object: expr, $method: expr, $arguments: expr, $return_buffer: expr) => {
        <dyn $service_trait as remote_trait_object::env::DispatchService<dyn $service_trait>>::dispatch(
            $object,
            $method,
            $arguments,
            $return_buffer,
        )
    };
}

#[macro_export]
macro_rules! service_id {
    ($service_trait: path) => {
        <dyn $service_trait as remote_trait_object::env::IdOfService<dyn $service_trait>>::id()
    };
}

/// These are set of functions that dispatcher / call stubs generated by the macro would call
pub mod service_context {
    pub use super::call::call;
    pub use super::call::delete;
    pub use super::dispatch::register;
}
