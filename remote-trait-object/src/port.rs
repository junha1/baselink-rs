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

pub mod client;
pub mod server;

use crate::ipc::{multiplex, IpcRecv, IpcSend};
use crate::service::{MethodId, PortDispatcher, Service, ServiceObjectId};
use parking_lot::RwLock;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Weak,
};

// This module implements two important communication models: Client and Server
//
// A calls B
// => A invokes B's method with Client.
// => B handles that call in Server dispatching the packet, and returns response.
// => A receives that response.
//
// Here servcie handler simply calls the dispatcher given by the port,
// whenever it receives a new inbound call.
//
// TODO: Introduce Rust async/await to support N concurrent calls where
// N > # of handler threads

pub type SlotId = u32;

const SLOT_CALL_OR_RETURN_INDICATOR: SlotId = 1000;
const DELETE_INDICATOR: MethodId = 1234;

const MULTIPLEX_INDEX_SERVER: usize = 0;
const MULTIPLEX_INDEX_CLIENT: usize = 1;

#[repr(C)]
#[derive(PartialEq, Debug)]
pub struct PacketHeader {
    pub slot: SlotId,
    pub handle: ServiceObjectId,
    pub method: MethodId,
}

impl PacketHeader {
    pub fn new(buffer: &[u8]) -> Self {
        unsafe { std::ptr::read(buffer.as_ptr().cast()) }
    }

    pub fn write(&self, buffer: &mut [u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(self, buffer.as_mut_ptr().cast(), 1);
        }
    }
}

#[test]
fn encoding_packet_header() {
    let ph1 = PacketHeader {
        slot: 0x1234,
        handle: ServiceObjectId {
            index: 0x8888,
        },
        method: 0x5678,
    };
    let mut buffer = vec![0 as u8; std::mem::size_of::<PacketHeader>()];
    ph1.write(&mut buffer);
    let ph2 = PacketHeader::new(&buffer);
    assert_eq!(ph2, ph1);
}

pub struct ServerOrClientForwarder;

impl multiplex::Forward for ServerOrClientForwarder {
    fn forward(data: &[u8]) -> usize {
        let header = PacketHeader::new(&data);
        if header.slot >= SLOT_CALL_OR_RETURN_INDICATOR {
            MULTIPLEX_INDEX_SERVER
        } else {
            MULTIPLEX_INDEX_CLIENT
        }
    }
}

pub trait Port: Send + Sync + 'static {
    // Client
    fn call(&self, handle: ServiceObjectId, method: MethodId, data: Vec<u8>) -> Vec<u8>;
    fn delete_request(&self, id: ServiceObjectId);

    // Server
    fn delete(&self, id: ServiceObjectId);
    /// Any service object involved in the return must have UNSET port
    fn dispatch(
        &self,
        handle: ServiceObjectId,
        method: MethodId,
        arguments: &[u8],
        return_buffer: std::io::Cursor<&mut Vec<u8>>,
    );
    /// handle_to_register's port must be UNSET.
    fn register(&self, handle_to_register: Arc<dyn Service>) -> ServiceObjectId;
}

/// Weak::new() is not implemented for ?Sized.
/// See https://github.com/rust-lang/rust/issues/50513
pub fn null_weak_port() -> Weak<dyn Port> {
    Weak::<BasicPort>::new() as Weak<dyn Port>
}

pub struct BasicPort {
    myself: RwLock<Weak<dyn Port>>,
    dispatcher: Arc<PortDispatcher>,
    termination: AtomicBool,
    client: client::Client,
}

impl Port for BasicPort {
    fn call(&self, handle: ServiceObjectId, method: MethodId, data: Vec<u8>) -> Vec<u8> {
        self.client.call(handle, method, data)
    }

    fn delete_request(&self, handle: ServiceObjectId) {
        // You don't need to send delete-me message if the whole Port is shutting down.
        if self.is_terminating() {
            return
        }
        self.client.delete(handle);
    }

    fn delete(&self, id: ServiceObjectId) {
        self.dispatcher.delete(id)
    }

    fn dispatch(
        &self,
        handle: ServiceObjectId,
        method: MethodId,
        arguments: &[u8],
        return_buffer: std::io::Cursor<&mut Vec<u8>>,
    ) {
        self.dispatcher.dispatch(handle, method, arguments, return_buffer)
    }

    fn register(&self, mut handle_to_register: Arc<dyn Service>) -> ServiceObjectId {
        Arc::get_mut(&mut handle_to_register).unwrap().get_handle_mut().port = self.myself.read().clone();
        self.dispatcher.register(handle_to_register)
    }
}

impl BasicPort {
    pub fn terminate(&self) {
        self.termination.store(true, Ordering::SeqCst);
    }

    pub fn is_terminating(&self) -> bool {
        self.termination.load(Ordering::SeqCst)
    }
}

pub struct PortInstance {
    /// _multiplexer must be dropped first
    _multiplexer: multiplex::Multiplexer,
    _server: server::Server,
    port: Arc<BasicPort>,
}

impl PortInstance {
    pub fn new<S: IpcSend + 'static, R: IpcRecv + 'static>(
        send: S,
        recv: R,
        dispatcher: Arc<PortDispatcher>,
        config: &crate::RtoConfig,
    ) -> Self {
        let (mut multiplex_ends, _multiplexer) =
            multiplex::Multiplexer::create::<ServerOrClientForwarder, S, R>(send, recv, 2, 256);

        let client = {
            let (send, recv) = multiplex_ends.pop().unwrap();
            client::Client::new(send, recv, config.call_slots as u32)
        };

        let port: Arc<BasicPort> = Arc::new(BasicPort {
            myself: RwLock::new(null_weak_port()),
            dispatcher,
            client,
            termination: AtomicBool::new(false),
        });
        let port_ = Arc::downgrade(&port) as Weak<dyn Port>;
        *port.myself.write() = port_;

        let _server = {
            let (send, recv) = multiplex_ends.pop().unwrap();
            server::Server::new(send, recv, port.clone(), config.server_threads, 128)
        };

        PortInstance {
            _multiplexer,
            _server,
            port,
        }
    }

    pub fn get_port(&self) -> Weak<dyn Port> {
        Arc::downgrade(&self.port) as Weak<dyn Port>
    }

    /// This will disable all service object garbage collection
    pub fn ready_to_terminate(&self) {
        self.port.terminate();
    }
}
