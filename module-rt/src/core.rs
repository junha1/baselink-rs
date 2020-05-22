use cbsb::execution::executee;
use cbsb::ipc::Ipc;
use parking_lot::Mutex;
use remote_trait_object::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ctx: &executee::Context<I>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

fn send<I: Ipc, T: Serialize>(ctx: &executee::Context<I>, data: &T) {
    ctx.ipc.as_ref().unwrap().send(&serde_cbor::to_vec(data).unwrap());
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

    pub fn export(&mut self, index: usize) -> Arc<dyn Service> {
        self.pool[index].take().unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct CoreMessageCreatePort {
    pub ipc_type: String,
    pub ipc_meta_argument: Vec<u8>,
    pub ipc_argument: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
pub enum CoreMessage {
    CreatePort(CoreMessageCreatePort),
    Terminate,
    Debug(Vec<u8>),
}

pub struct Core<I: Ipc> {
    process_executee: executee::Context<I>,
    export_pool: Arc<Mutex<ExportingServicePool>>,
}

impl<I: Ipc> Core<I> {
    pub fn new(process_arg: Vec<String>) -> Self {
        let ctx = executee::start::<I>(process_arg);

        let id_map: IdMap = recv(&ctx);
        let init: Vec<u8> = recv(&ctx);
        let exports: Vec<(String, Vec<u8>)> = recv(&ctx);
        let x: Vec<(&str, &[u8])> = exports.iter().map(|(x, y)| (x as &str, y as &[u8])).collect();

        Core {
            process_executee: ctx,
            export_pool: Arc::new(Mutex::new(ExportingServicePool::new(&x))),
        }
    }

    pub fn get_message(&self) -> CoreMessage {
        recv(&self.process_executee)
    }
}
