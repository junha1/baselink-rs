use super::*;
use std::cell::RefCell;
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
use std::sync::Arc;
use std::io::{Write, Read};
use parking_lot::RwLock;
use std::marker::PhantomData;
use std::io::Cursor;

pub struct DomainSocketSend {
    socket: Arc<RwLock<UnixStream>>
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {
        let mut guard = self.socket.write();
        let size: [u8; 8] = data.len().to_be_bytes();
        // these two operations must be atomic
        assert_eq!(guard.write(&size).unwrap(), 8);
        assert_eq!(guard.write(data).unwrap(), data.len());
    }
}

pub struct DomainSocketRecv {
    socket: Arc<RwLock<UnixStream>>,
    buffer: RefCell<Vec<u8>>,
}

impl IpcRecv for DomainSocketRecv {
    type Terminator = Terminator;

    /// Note that DomainSocketRecv is !Sync, so this is guaranteed to be mutual exclusive.
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        let mut guard = self.socket.write();
        guard.set_read_timeout(timeout).unwrap();
        let count = guard.read(&mut self.buffer.borrow_mut()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok(Result::<usize, RecvError>::Err(RecvError::TimeOut))
            } else {
                Err(e)
            }
        }).map(|x| Ok(x)).unwrap()?;
        let size_buffer: [u8; 8] = std::convert::TryInto::try_into(&self.buffer.borrow()[0..8]).unwrap();
        
        let size = usize::from_be_bytes(size_buffer);
        let size_read = count - 8;

        let mut result: Vec<u8> = Vec::with_capacity(size);
        result.extend_from_slice(&self.buffer.borrow()[8..]);

        guard.read_exact(&mut result[size_read..]).map_err(|e| {
            if e.kind() == std::io::ErrorKind::TimedOut {
                Ok(Result::<usize, RecvError>::Err(RecvError::TimeOut))
            } else {
                Err(e)
            }
        }).map(|_| Ok(())).unwrap()?;
        Ok(result)
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.socket.clone())
    }
}

pub struct Terminator(Arc<RwLock<UnixStream>>);

impl Terminate for Terminator {
    fn terminate(&self) {
        if let Err(e) = (self.0).read().shutdown(std::net::Shutdown::Both) {
            assert_eq!(e.kind(), std::io::ErrorKind::NotConnected);
        }
    }
}

pub struct DomainSocket {
    send: DomainSocketSend,
    recv: DomainSocketRecv
}


impl IpcSend for DomainSocket {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for DomainSocket {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.recv.recv(timeout)
    }

    fn create_terminator(&self) -> Self::Terminator {
        self.recv.create_terminator()
    }
}

impl Ipc for DomainSocket {
    fn arguments_for_both_ends() -> (Vec<u8>, Vec<u8>) {
        let address_server = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        let address_client = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        (
            serde_cbor::to_vec(&(true, &address_server, &address_client)).unwrap(),
            serde_cbor::to_vec(&(false, &address_client, &address_server)).unwrap(),
        )
    }

    type SendHalf = DomainSocketSend;
    type RecvHalf = DomainSocketRecv;

    fn new(data: Vec<u8>) -> Self {
        let (am_i_server, address_src, address_dst): (bool, String, String) = serde_cbor::from_slice(&data).unwrap();
        
        let stream = if am_i_server {
            let listener = UnixListener::bind(&address_src).unwrap();
            listener.incoming().nth(0).unwrap().unwrap()
        } else {
            (|| {
                for _ in 0..100 {
                    if let Ok(stream) = UnixStream::connect(&address_dst) {
                        return stream
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                panic!("Failed to establish domain socket within a timeout")
            })()
        };

        let socket = Arc::new(RwLock::new(stream));
        DomainSocket {
            send: DomainSocketSend {
                socket: socket.clone(),
            },
            recv: DomainSocketRecv {
                socket,
                buffer: RefCell::new(vec![0; 1024 * 8 + 8]),
            },
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }

}