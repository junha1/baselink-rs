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

/// NullServive is the only actual service trait that remote-trait-object provides by default.
/// It will be useful when you want to establish a remote-trait-object connection with with_initial_service(),
/// but such initial service is needed by only one side.
pub trait NullService: Service {}

pub fn create_null_service() -> Box<dyn NullService> {
    Box::new(NullServiceImpl)
}

struct NullServiceImpl;

impl NullService for NullServiceImpl {}

impl Service for NullServiceImpl {}

// Contents below are something that would have been generated by macro.
// They are slightly different from the actual expansion result of NullService (which happens to succeed), since the macro
// doesn't take account of such special case.

pub struct NullServiceBoxDispatcher {}
impl NullServiceBoxDispatcher {
    fn new(_object: Box<dyn NullService>) -> Self {
        Self {}
    }
}
impl crate::macro_env::Dispatch for NullServiceBoxDispatcher {
    fn dispatch_and_call(&self, _method: crate::macro_env::MethodId, _args: &[u8]) -> Vec<u8> {
        panic!("Invalid remote-trait-object call. Fatal Error.")
    }
}
impl crate::macro_env::IntoService<dyn NullService> for Box<dyn NullService> {
    fn into_service(self) -> crate::macro_env::ServiceToRegister {
        crate::macro_env::ServiceToRegister::new(std::sync::Arc::new(NullServiceBoxDispatcher::new(self)))
    }
}
pub struct NullServiceArcDispatcher {}
impl NullServiceArcDispatcher {
    fn new(_object: std::sync::Arc<dyn NullService>) -> Self {
        Self {}
    }
}
impl crate::macro_env::Dispatch for NullServiceArcDispatcher {
    fn dispatch_and_call(&self, _method: crate::macro_env::MethodId, _args: &[u8]) -> Vec<u8> {
        panic!("Invalid remote-trait-object call. Fatal Error.")
    }
}
impl crate::macro_env::IntoService<dyn NullService> for std::sync::Arc<dyn NullService> {
    fn into_service(self) -> crate::macro_env::ServiceToRegister {
        crate::macro_env::ServiceToRegister::new(std::sync::Arc::new(NullServiceArcDispatcher::new(self)))
    }
}
pub struct NullServiceRwLockDispatcher {}
impl NullServiceRwLockDispatcher {
    fn new(_object: std::sync::Arc<parking_lot::RwLock<dyn NullService>>) -> Self {
        Self {}
    }
}
impl crate::macro_env::Dispatch for NullServiceRwLockDispatcher {
    fn dispatch_and_call(&self, _method: crate::macro_env::MethodId, _args: &[u8]) -> Vec<u8> {
        panic!("Invalid remote-trait-object call. Fatal Error.")
    }
}
impl crate::macro_env::IntoService<dyn NullService> for std::sync::Arc<parking_lot::RwLock<dyn NullService>> {
    fn into_service(self) -> crate::macro_env::ServiceToRegister {
        crate::macro_env::ServiceToRegister::new(std::sync::Arc::new(NullServiceRwLockDispatcher::new(self)))
    }
}
#[derive(Debug)]
pub struct NullServiceRemote {
    handle: crate::macro_env::Handle,
}
impl NullService for NullServiceRemote {}
impl crate::macro_env::Service for NullServiceRemote {}
impl crate::macro_env::ImportRemote<dyn NullService> for Box<dyn NullService> {
    fn import_remote(
        port: std::sync::Weak<dyn crate::macro_env::Port>,
        handle: crate::macro_env::HandleToExchange,
    ) -> Self {
        Box::new(NullServiceRemote {
            handle: crate::macro_env::Handle::careful_new(handle, port),
        })
    }
}
impl crate::macro_env::ImportRemote<dyn NullService> for std::sync::Arc<dyn NullService> {
    fn import_remote(
        port: std::sync::Weak<dyn crate::macro_env::Port>,
        handle: crate::macro_env::HandleToExchange,
    ) -> Self {
        std::sync::Arc::new(NullServiceRemote {
            handle: crate::macro_env::Handle::careful_new(handle, port),
        })
    }
}
impl crate::macro_env::ImportRemote<dyn NullService> for std::sync::Arc<parking_lot::RwLock<dyn NullService>> {
    fn import_remote(
        port: std::sync::Weak<dyn crate::macro_env::Port>,
        handle: crate::macro_env::HandleToExchange,
    ) -> Self {
        std::sync::Arc::new(parking_lot::RwLock::new(NullServiceRemote {
            handle: crate::macro_env::Handle::careful_new(handle, port),
        }))
    }
}
