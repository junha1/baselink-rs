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

use crate::service::{HandleInstance, MethodId};
use crate::PacketHeader;
use std::io::Cursor;

impl HandleInstance {
    pub fn call<S: serde::Serialize, D: serde::de::DeserializeOwned>(&self, method: MethodId, args: &S) -> D {
        #[cfg(statistics)]
        {
            crate::statistics::CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        super::serde_support::port_thread_local::set_port(self.port.clone());
        let mut buffer: Vec<u8> = Vec::new();
        buffer.resize(std::mem::size_of::<PacketHeader>(), 0 as u8);
        serde_cbor::to_writer(
            {
                let mut c = Cursor::new(&mut buffer);
                c.set_position(std::mem::size_of::<PacketHeader>() as u64);
                c
            },
            &args,
        )
        .unwrap();
        let result = self.port.upgrade().unwrap().call(self.id, method, buffer);
        let v = serde_cbor::from_reader(&result[std::mem::size_of::<PacketHeader>()..]).unwrap();
        super::serde_support::port_thread_local::remove_port();
        v
    }

    pub fn delete(&self) {
        #[cfg(statistics)]
        {
            crate::statistics::DELETE_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        }
        self.port.upgrade().unwrap().delete_request(self.id);
    }
}
