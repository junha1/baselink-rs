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

// Mock functions for the dispatcher / method stubs

use crate as remote_trait_object;

use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use remote_trait_object::*;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

thread_local!(static INSTANCE_KEY: Cell<u32> = Cell::new(0));

pub fn set_key(key: u32) {
    INSTANCE_KEY.with(|k| {
        assert_eq!(k.get(), 0);
        k.set(key);
    })
}

fn get_key() -> u32 {
    INSTANCE_KEY.with(|k| {
        assert_ne!(k.get(), 0);
        k.get()
    })
}

type LogQueue = VecDeque<Vec<u8>>;
static LOG: OnceCell<RwLock<HashMap<u32, LogQueue>>> = OnceCell::new();
type ServiceLogQueue = VecDeque<Arc<dyn Service>>;
static SERVICE_LOG: OnceCell<RwLock<HashMap<u32, ServiceLogQueue>>> = OnceCell::new();

fn push_log(s: Vec<u8>) {
    let mut guard = LOG.get_or_init(Default::default).write();
    if let Some(queue) = guard.get_mut(&get_key()) {
        queue.push_back(s)
    } else {
        guard.insert(get_key(), VecDeque::new());
        drop(guard);
        push_log(s);
    }
}
pub fn pop_log() -> Vec<u8> {
    LOG.get_or_init(Default::default).write().remove(&get_key()).unwrap().pop_front().unwrap()
}

fn push_service_log(s: Arc<dyn Service>) {
    let mut guard = SERVICE_LOG.get_or_init(Default::default).write();
    if let Some(queue) = guard.get_mut(&get_key()) {
        queue.push_back(s)
    } else {
        guard.insert(get_key(), VecDeque::new());
        drop(guard);
        push_service_log(s);
    }
}
pub fn pop_service_log() -> Arc<dyn Service> {
    SERVICE_LOG.get_or_init(Default::default).write().remove(&get_key()).unwrap().pop_front().unwrap()
}

// We define this instead of std::default::Default because of the orphan rule.
pub trait TestDefault {
    fn default() -> Self;
}

pub fn register(port_id: PortId, handle_to_register: Arc<dyn Service>) -> HandleInstance {
    push_log(serde_cbor::to_vec(&("register", port_id)).unwrap());
    push_service_log(handle_to_register);
    Default::default()
}
pub fn call<S: serde::Serialize + std::fmt::Debug, D: serde::de::DeserializeOwned + TestDefault>(
    handle: &HandleInstance,
    method: MethodId,
    args: &S,
) -> D {
    push_log(serde_cbor::to_vec(&("call", handle, method, args)).unwrap());
    TestDefault::default()
}
pub fn delete(handle: &HandleInstance) {
    push_log(serde_cbor::to_vec(&("delete", handle)).unwrap());
}
