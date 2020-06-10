use remote_trait_object::*;
use serde::{Deserialize, Serialize};
use std::sync::Weak;

#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct HandleExchange {
    /// Id of exporter (same as that in Config)
    pub exporter: String,
    /// Id of importer (same as that in Config)
    pub importer: String,
    /// Handles. Importer must cast these to Arc<dyn SomeHandle> itself.
    pub handles: Vec<HandleToExchange>,
    /// Opaque argument
    pub argument: Vec<u8>,
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// kind of this module. Per-binary
    pub kind: String,
    /// id of this instance of module. Per-instance, Per-appdescriptor
    pub id: String,
    /// key of this instance of module. Per-instance, Per-execution, Per-node
    pub key: u32,
    /// Arguments given to this module.
    pub args: Vec<u8>,
}

pub trait Bootstrap {
    fn new(config: &Config) -> Self;
    fn export(&mut self, ports: &mut super::PortTable) -> Vec<HandleExchange>;
    fn import(&mut self, port: Weak<dyn Port>, exchange: HandleExchange);
    fn debug(&self, arg: &[u8]) -> Vec<u8>;
}
