use cbsb::ipc::Ipc;
use parking_lot::Mutex;
use remote_trait_object::{HandleInstance, Port as RtoPort, PortId, Service};
use std::sync::Arc;
use std::thread;

#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1_000_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ipc: &I) -> T {
    serde_cbor::from_slice(&ipc.recv(None).unwrap()).unwrap()
}

fn send<I: Ipc, T: serde::Serialize>(ipc: &I, data: &T) {
    ipc.send(&serde_cbor::to_vec(data).unwrap());
}

struct Bootstrap<I: Ipc> {
    export: Vec<usize>,
    import: Vec<String>,
    ipc_meta: I,
}

impl<I: Ipc> Bootstrap<I> {
    fn new(ipc_meta: I) -> Self {
        unimplemented!()
    }
}

pub struct LinkedPort<I: Ipc> {
    rto_port: RtoPort,
    ipc_meta: I,
}

pub struct Port<I: Ipc + 'static> {
    bootsrapping: thread::JoinHandle<Bootstrap<I>>,
}

impl<I: Ipc + 'static> Port<I> {
    pub fn new(port_id: PortId, ipc_meta_argument: Vec<u8>) -> Self {
        let ipc_meta = I::new(ipc_meta_argument);
        let bootsrapping = thread::spawn(move || Bootstrap::<I>::new(ipc_meta));
        Port {
            bootsrapping,
        }
    }

    pub fn link(
        self,
        ipc_common_argument: Vec<u8>,
        exporting_service_pool: Arc<Mutex<crate::core::ExportingServicePool>>,
        importer: Box<dyn FnOnce(Vec<(String, HandleInstance)>)>,
    ) -> LinkedPort<I> {
        let Bootstrap {
            export,
            import,
            ipc_meta,
        } = self.bootsrapping.join().unwrap();
        let handles: Vec<HandleInstance> = serde_cbor::from_slice(&ipc_meta.recv(Some(TIMEOUT)).unwrap()).unwrap();
        assert_eq!(handles.len(), import.len());
        importer(import.into_iter().zip(handles.into_iter()).collect());

        let ipc_common = I::new(ipc_common_argument);

        let mut pool_guard = exporting_service_pool.lock();
        let services_to_export: Vec<Arc<dyn Service>> = export.into_iter().map(|i| pool_guard.export(i)).collect();

        let dispatcher = Arc::new(remote_trait_object::PortDispatcher::new(0, 128));
        let config = recv(&ipc_common);

        let (send, recv) = ipc_common.split();
        RtoPort::new(send, recv, 0, dispatcher, 0, &config);

        unimplemented!()
    }
}
