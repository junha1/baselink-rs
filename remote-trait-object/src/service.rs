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

use super::Port;
pub use dispatch::PortDispatcher;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Weak};

pub type MethodId = u32;
pub type TraitId = u16;
pub type InstanceId = u16;

// We avoid using additional space with Option<>, by these.
pub const UNDECIDED_INDEX: InstanceId = std::u16::MAX;

/// This struct represents an index to a service object in port server's registry
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ServiceObjectId {
    pub(crate) index: InstanceId,
}

pub struct HandleInstance {
    pub(crate) id: ServiceObjectId,
    // The port this handle belongs to
    pub(crate) port: Weak<dyn Port>,
}

#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandleToExchange(ServiceObjectId);

impl HandleToExchange {
    pub fn from_id(id: ServiceObjectId) -> Self {
        Self(id)
    }
}

/// Handle created by this must be local-only
impl Default for HandleInstance {
    fn default() -> Self {
        let x: Weak<crate::BasicPort> = Weak::new(); // Rust doesn't allow Weak::<dyn Port>::new()
        HandleInstance {
            id: ServiceObjectId {
                index: UNDECIDED_INDEX,
            },
            port: x,
        }
    }
}

impl std::fmt::Debug for HandleInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.id.fmt(f)
    }
}

impl HandleInstance {
    /// You (module implementor) should not call this!
    pub fn careful_new(port: Weak<dyn Port>, id: HandleToExchange) -> Self {
        HandleInstance {
            id: id.0,
            port,
        }
    }

    /// You (module implementor) should not call this!
    pub fn careful_set(&mut self, port: Weak<dyn Port>) {
        self.port = port;
    }

    /// You (module implementor) should not call this!
    pub fn careful_export(&self) -> HandleToExchange {
        HandleToExchange(self.id)
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
    fn import(port: Weak<dyn Port>, handle: HandleToExchange) -> Arc<T>;
}

pub trait ExportService<T: ?Sized + Service> {
    fn export(port: Weak<dyn Port>, object: Arc<T>) -> HandleToExchange;
}

pub trait DispatchService<T: ?Sized + Service> {
    fn dispatch(object: &T, method: MethodId, arguments: &[u8], return_buffer: std::io::Cursor<&mut Vec<u8>>);
}

pub trait IdOfService<T: ?Sized + Service> {
    fn id() -> TraitId;
}

#[macro_export]
macro_rules! service_export {
    ($service_trait: path, $port: expr, $arg: expr) => {{
        <dyn $service_trait as remote_trait_object::ExportService<dyn $service_trait>>::export($port, $arg)
    }};
}

#[macro_export]
macro_rules! service_import {
    ($service_trait: path, $port: expr, $arg: expr) => {
        <dyn $service_trait as remote_trait_object::ImportService<dyn $service_trait>>::import($port, $arg)
    };
}

#[macro_export]
macro_rules! service_dispatch {
    ($service_trait: path, $object: expr, $method: expr, $arguments: expr, $return_buffer: expr) => {
        <dyn $service_trait as remote_trait_object::DispatchService<dyn $service_trait>>::dispatch(
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
        <dyn $service_trait as remote_trait_object::IdOfService<dyn $service_trait>>::id()
    };
}
