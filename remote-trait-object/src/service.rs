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

pub mod export_import;
pub mod id;
mod null;
pub mod remote;
pub mod serde_support;

use crate::forwarder::ServiceObjectId;
use crate::port::{Port, RemoteObjectId};
use serde::{Deserialize, Serialize};
use std::sync::Weak;

pub use null::{create_null_service, NullService};
pub type MethodId = u32;

/// This represents transportable identifier of the service object
/// and should be enough to construct a handle along with the pointer to the port
/// which this service belong to
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct HandleToExchange(pub(crate) ServiceObjectId);

/// Remote service will carry this.
#[derive(Debug)]
pub struct Handle {
    service_id: ServiceObjectId,
    remote_id: RemoteObjectId,
    port: Weak<dyn Port>,
}

impl Handle {
    pub fn new(handle: HandleToExchange, port: Weak<dyn Port>) -> Self {
        Handle {
            service_id: handle.0,
            remote_id: port.upgrade().unwrap().register_remote(handle),
            port,
        }
    }
}

/// Exporter sides's interface to the service object. This will be implemented
/// by each service trait's unique wrapper in the macro
pub trait Dispatch: Send + Sync {
    fn dispatch_and_call(&self, method: MethodId, args: &[u8]) -> Vec<u8>;
}

impl<F> Dispatch for F
where
    F: Fn(MethodId, &[u8]) -> Vec<u8> + Send + Sync,
{
    fn dispatch_and_call(&self, method: MethodId, args: &[u8]) -> Vec<u8> {
        self(method, args)
    }
}

/// All service trait must implement this.
/// This trait serves as a mere marker trait with two bounds
pub trait Service: Send + Sync {}

/// A serde de/serialization format that will be used for a service.
pub trait SerdeFormat {
    fn to_vec<S: serde::Serialize>(s: &S) -> Result<Vec<u8>, ()>;
    fn from_slice<D: serde::de::DeserializeOwned>(data: &[u8]) -> Result<D, ()>;
}

/// In most case the format isn't important because the users won't see the raw data directly anyway.
/// Thus we provide a default format for the macro.
pub struct Cbor;

impl SerdeFormat for Cbor {
    fn to_vec<S: serde::Serialize>(s: &S) -> Result<Vec<u8>, ()> {
        serde_cbor::to_vec(s).map_err(|_| ())
    }

    fn from_slice<D: serde::de::DeserializeOwned>(data: &[u8]) -> Result<D, ()> {
        serde_cbor::from_slice(data).map_err(|_| ())
    }
}
