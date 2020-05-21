use remote_trait::*;
use serde::{Deserialize, Serialize};

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
/// We assume that there could be at most one link for a pair of modules in this exchange phase,
/// so no information about PortId is carried.
pub trait HandlePreset {
    fn export() -> Vec<HandleExchange>;
    fn import(exchange: HandleExchange);
}

pub fn find_port_id(id: &str) -> Result<PortId, ()> {
    let table = global::get().read();
    Ok(*table.map.iter().find(|&(_, (name, ..))| name == id).ok_or(())?.0)
}
