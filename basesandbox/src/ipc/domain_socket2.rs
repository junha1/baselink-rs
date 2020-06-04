use super::*;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::Arc;

pub struct DomainSocketSend {
    socket: Arc<RwLock<UnixStream>>,
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
}

impl IpcRecv for DomainSocketRecv {
    type Terminator = Terminator;

    /// Note that DomainSocketRecv is !Sync, so this is guaranteed to be mutual exclusive.
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        let mut guard = self.socket.write();
        guard.set_read_timeout(timeout).unwrap();

        fn recv_helper(buf: &mut [u8], stream: &mut UnixStream) -> Result<(), RecvError> {
            let r = stream.read_exact(buf);
            match r {
                Ok(_) => Ok(()),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::TimedOut => Err(RecvError::TimeOut),
                    std::io::ErrorKind::UnexpectedEof => Err(RecvError::Termination),
                    e => panic!(e),
                },
            }
        }

        let mut size_buf = [0 as u8; 8];
        recv_helper(&mut size_buf, &mut *guard)?;
        let size = usize::from_be_bytes(size_buf);

        let mut result: Vec<u8> = Vec::with_capacity(size);
        // TODO: avoid useless initialization
        result.resize(size, 0);

        if size == 0 {
            panic!()
        }
        recv_helper(&mut result[0..], &mut *guard)?;
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
    recv: DomainSocketRecv,
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

        stream.set_write_timeout(None).unwrap();
        stream.set_nonblocking(false).unwrap();
        let socket = Arc::new(RwLock::new(stream));

        DomainSocket {
            send: DomainSocketSend {
                socket: socket.clone(),
            },
            recv: DomainSocketRecv {
                socket,
            },
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}

#[test]
fn f123() {
    let (a1, a2) = DomainSocket::arguments_for_both_ends();

    let t = std::thread::spawn(|| DomainSocket::new(a2));
    let s1 = DomainSocket::new(a1);
    let s2 = t.join().unwrap();

    let huge_data = {
        let mut v = Vec::new();
        for i in 0..300000 {
            v.push((i % 255) as u8)
        }
        v
    };

    s2.send(&huge_data);

    let r = s1.recv(None).unwrap();

    s1.send(&huge_data);
    let r = s2.recv(None).unwrap();

    println!("DONE");
}
