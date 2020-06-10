mod context;
mod control_loop;
pub mod prelude;

pub use cbsb::ipc::unix_socket::DomainSocket as DefaultIpc;
pub use context::{Bootstrap, Config, HandleExchange};
pub use control_loop::run_control_loop;

pub type PortTable = std::collections::HashMap<String, remote_trait_object::PortInstance>;
