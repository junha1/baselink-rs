#![allow(dead_code, unused_variables)]

use remote_trait::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// TODO: Replace this with LinkBootstrapping.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct HandleExchange {
    /// Id of exporter (same as that in Config)
    pub exporter: String,
    /// Id of importer (same as that in Config)
    pub importer: String,
    /// Handles. Importer must cast these to Arc<dyn SomeHandle> itself.
    pub handles: Vec<HandleInstance>,
    /// Opaque argument
    pub argument: Vec<u8>,
}

/// TODO: Replace this with LinkBootstrapping.
/// We assume that there could be at most one link for a pair of modules in this exchange phase,
/// so no information about PortId is carried.
pub trait HandlePreset {
    fn export() -> Vec<HandleExchange>;
    fn import(exchange: HandleExchange);
}

/// TODO: Replace this with LinkBootstrapping
pub fn find_port_id(id: &str) -> Result<PortId, ()> {
    let table = global::get().read();
    let keys: Vec<String> = (*table).map.iter().map(|x| (x.1).0.clone()).collect();
    Ok(*table.map.iter().find(|&(_, (name, ..))| name == id).ok_or(())?.0)
}

pub fn create_service_to_export(method_name: &str, argument: &[u8]) -> Arc<dyn Service> {
    panic!()
}

pub struct ExportingServicePool {
    pool: Vec<Option<Arc<dyn Service>>>,
}

impl ExportingServicePool {
    pub fn new(ctors: &[(&str, &[u8])]) -> Self {
        let pool = ctors.iter().map(|(method, arg)| Some(create_service_to_export(method, arg))).collect();
        ExportingServicePool {
            pool,
        }
    }

    pub fn export(&mut self, port_id: PortId, index: usize) -> Vec<u8> {
        let service = self.pool[index].take().unwrap();
        let handle = env::service_context::register(port_id, service);
        serde_cbor::to_vec(&handle).unwrap()
    }
}
