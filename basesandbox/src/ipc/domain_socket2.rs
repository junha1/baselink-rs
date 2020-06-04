use super::*;
use parking_lot::Mutex;
use std::cell::RefCell;
use std::io::{Read, Write};
use mio::net::{UnixStream, UnixListener};
use std::sync::Arc;
use std::marker::PhantomData;
use mio::{Events, Interest, Poll, Token};
use mio::net::UdpSocket;
const POLL_TOKEN: Token = Token(0);

pub struct DomainSocketSend {
    socket: Arc<Mutex<UnixStream>>,
    poll: Mutex<Poll>,
    _prevent_sync: PhantomData<std::cell::Cell<()>>
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {
        let mut guard = self.socket.lock();
        let size: [u8; 8] = data.len().to_be_bytes();
        
        let mut send_helper= |buf: &[u8]| {
            let mut sent = 0;
            let mut events = Events::with_capacity(1);
            while sent < data.len() {
                self.poll.lock().poll(&mut events, None).unwrap();
                assert_eq!(events.iter().nth(0).unwrap().token(), POLL_TOKEN, "Invalid socket event");
                match guard.write(buf) {
                    Ok(x) => sent += x,
                    Err(e) => {
                        match e.kind() {
                            std::io::ErrorKind::WouldBlock => (),
                            _ => panic!("Failed to send")
                        }
                    },
                }
            }
            assert_eq!(sent, data.len());
        };
        send_helper(&size);
        send_helper(&data);
    }
}

pub struct DomainSocketRecv {
    socket: Arc<Mutex<UnixStream>>,
    poll: Mutex<Poll>,
    _prevent_sync: PhantomData<std::cell::Cell<()>>
}

impl IpcRecv for DomainSocketRecv {
    type Terminator = Terminator;

    /// Note that DomainSocketRecv is !Sync, so this is guaranteed to be mutual exclusive.
    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        fn recv_helper(buf: &mut [u8], stream: &mut UnixStream) -> Result<bool, RecvError> {
            let r = stream.read_exact(buf);
            match r {
                Ok(_) => Ok(true),
                Err(e) => match e.kind() {
                    std::io::ErrorKind::UnexpectedEof => Err(RecvError::Termination),
                    std::io::ErrorKind::WouldBlock => Ok(false), // spurious wakeup
                    e => panic!(e),
                },
            }
        }
        let mut events = Events::with_capacity(1);

        let size;
        loop {
            self.poll.lock().poll(&mut events, timeout).map_err(|_| RecvError::TimeOut)?;
            assert_eq!(events.iter().nth(0).unwrap().token(), POLL_TOKEN, "Invalid socket event");
            let mut guard = self.socket.lock();
            let mut size_buf = [0 as u8; 8];
            if recv_helper(&mut size_buf, &mut *guard)? {
                size = usize::from_be_bytes(size_buf);
                break;
            }
        }

        let mut result: Vec<u8> = Vec::with_capacity(size);
        // TODO: avoid useless initialization
        result.resize(size, 0);
        if size == 0 {
            panic!()
        }

        loop {
            let mut guard = self.socket.lock();
            if recv_helper(&mut result[0..], &mut *guard)? {
                break;
            }
            // we wait later this time, since the preceeding polling might be upto this recv.
            self.poll.lock().poll(&mut events, timeout).map_err(|_| RecvError::TimeOut)?;
            assert_eq!(events.iter().nth(0).unwrap().token(), POLL_TOKEN, "Invalid socket event");
        }
        Ok(result)
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.socket.clone())
    }
}

pub struct Terminator(Arc<Mutex<UnixStream>>);

impl Terminate for Terminator {
    fn terminate(&self) {
        if let Err(e) = (self.0).lock().shutdown(std::net::Shutdown::Both) {
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

        // We use spinning for the connection establishment
        let mut stream = if am_i_server {
            let listener = UnixListener::bind(&address_src).unwrap();
            (|| {
                for _ in 0..100 {
                    if let Ok(stream) = listener.accept() {
                        return stream
                    }
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                panic!("Failed to establish domain socket within a timeout")
            })().0
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


        let mut poll1 = Poll::new().unwrap();
        poll1.registry()
        .register(&mut stream, POLL_TOKEN, Interest::READABLE).unwrap();

        let mut poll2 = Poll::new().unwrap();
        poll2.registry()
        .register(&mut stream, POLL_TOKEN, Interest::READABLE).unwrap();

        let socket = Arc::new(Mutex::new(stream));

        DomainSocket {
            send: DomainSocketSend {
                socket: socket.clone(),
                poll: Mutex::new(poll1),
                _prevent_sync: PhantomData
            },
            recv: DomainSocketRecv {
                socket,
                poll: Mutex::new(poll2),
                _prevent_sync: PhantomData
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
}
