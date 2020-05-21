use remote_trait::{context_provider, InstanceKey};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// kind of this module. Per-binary
    pub kind: String,
    /// id of this instance of module. Per-instance, Per-appdescriptor
    pub id: String,
    /// key of this instance of module. Per-instance, Per-execution, Per-node
    pub key: InstanceKey,
    /// Arguments given to this module.
    pub args: Vec<u8>,
}

context_provider! {Config}
pub fn get_module_config() -> &'static Config {
    context_provider_mod::get()
}

pub(crate) fn set_module_config(ctx: Config) {
    context_provider_mod::set(ctx)
}

pub(crate) fn remove_module_config() {
    context_provider_mod::remove()
}
