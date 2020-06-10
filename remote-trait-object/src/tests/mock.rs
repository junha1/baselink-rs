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

use crate::*;
use once_cell::sync::OnceCell;
use parking_lot::RwLock;
use std::cell::Cell;
use std::collections::HashMap;
use std::collections::VecDeque;
use std::sync::Arc;

type DeleteQueue = VecDeque<ServiceObjectId>;
type DeleteRequestQueue = VecDeque<ServiceObjectId>;
type RegisterQueue = VecDeque<Arc<dyn Service>>;

const TEST_OBJECT_ID: ServiceObjectId = ServiceObjectId {
    index: 1234,
};

#[derive(Default)]
pub struct TestPort {
    log_delete: RwLock<DeleteQueue>,
    log_delete_request: RwLock<DeleteQueue>,
    log_service: RwLock<RegisterQueue>,
}

impl Port for TestPort {
    fn call(&self, handle: ServiceObjectId, method: MethodId, data: Vec<u8>) -> Vec<u8> {
        panic!("Dummy Call")
    }

    fn delete_request(&self, id: ServiceObjectId) {
        self.log_delete_request.write().push_back(id);
    }

    fn delete(&self, id: ServiceObjectId) {
        self.log_delete.write().push_back(id);
    }

    fn dispatch(
        &self,
        handle: ServiceObjectId,
        method: MethodId,
        arguments: &[u8],
        return_buffer: std::io::Cursor<&mut Vec<u8>>,
    ) {
        panic!()
    }

    fn register(&self, handle_to_register: Arc<dyn Service>) -> ServiceObjectId {
        self.log_service.write().push_back(handle_to_register);
        TEST_OBJECT_ID
    }
}
