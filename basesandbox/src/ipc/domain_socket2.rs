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
use std::thread;
use crossbeam::channel::{Receiver, Sender, self};

fn send_routine(queue: Receiver<Vec<u8>>, write_signal: Receiver<Result<(), ()>>, socket: Arc<Mutex<UnixStream>>) -> Result<(), ()> {
    let mut send_helper= |buf: &[u8]| {
        let mut sent = 0;
        let mut events = Events::with_capacity(1);
        while sent < buf.len() {
            match socket.lock().write(buf) {
                Ok(x) => sent += x,
                Err(e) => {
                    match e.kind() {
                        // spurious wakeup
                        std::io::ErrorKind::WouldBlock => write_signal.recv().map_err(|_| ())?, 
                        _ => panic!("Failed to send"),
                    }
                },
            }
        }
        assert_eq!(sent, buf.len());
        Ok(())
    };
    
    loop {
        let x = queue.recv().unwrap();
        if x.is_empty() {
            return Err(());
        }
        let size: [u8; 8] = x.len().to_be_bytes();
        send_helper(&size)?;
        send_helper(&x)?;
    }
}

fn recv_routine(queue: Sender<Vec<u8>>, read_signal: Receiver<Result<(), ()>>, socket: Arc<Mutex<UnixStream>>) -> Result<(), ()>  {
    let mut recv_helper = |buf: &mut [u8]| {
        let r = socket.lock().read_exact(buf);
        match r {
            Ok(_) => Ok(true),
            Err(e) => match e.kind() {
                std::io::ErrorKind::UnexpectedEof => Err(RecvError::Termination),
                std::io::ErrorKind::WouldBlock => Ok(false), // spurious wakeup
                e => panic!(e),
            },
        }
    };
    let mut events = Events::with_capacity(1);

    let size;
    loop {
        read_signal.recv().unwrap()?;
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

fn poll_routine(write_signal: Sender<()>, recv_signal: Sender<()>, mut poll: Poll) {
    // TODO: does the capacity matter?
    let mut events = Events::with_capacity(100);
    loop {
        poll.poll(&mut events, None).unwrap();
        for event in events.iter() {
            assert_eq!(events.iter().nth(0).unwrap().token(), POLL_TOKEN, "Invalid socket event");
            // termintation
            if event.is_error() { 
                return;
            }
            if event.is_writable() {
                write_signal.send(()).unwrap();
            }
            if event.is_readable() {
                recv_signal.send(()).unwrap();
            }
        }
    }
}

struct SocketInternal {
    send_thread: Option<thread::JoinHandle<()>>,
    poll_thread: Option<thread::JoinHandle<()>>,
}

impl SocketInternal {
    fn create(mut socket: UnixStream) -> (DomainSocketSend, DomainSocketRecv, Arc<Self>) {
        let mut poll = Poll::new().unwrap();
        poll.registry()
        .register(&mut socket, POLL_TOKEN, Interest::WRITABLE.add(Interest::READABLE)).unwrap();

        let socket = Arc::new(Mutex::new(socket));
        let (send_queue_send, send_queue_recv) = channel::unbounded();
        let (write_signal_send, write_signal_recv) = channel::unbounded();
        let (read_signal_send, read_signal_recv) = channel::unbounded();

        let socket_ = socket.clone();
        let send_thread = Some(thread::spawn(|| {
            send_routine(send_queue_recv, write_signal_recv, socket_);
        }));
        
        let poll_thread = Some(thread::spawn(|| {
            poll_routine(write_signal_send, read_signal_send, poll);
        }));

        (
            DomainSocketSend{queue: send_queue_send},
            DomainSocketRecv{

            },
        SocketInternal {
            send_thread,
            poll_thread
        })
    }
}

impl Drop for SocketInternal{
    fn drop(&mut self) {
        self.send_thread.take().unwrap().join().unwrap();
        self.poll_thread.take().unwrap().join().unwrap();
    }
}


pub struct DomainSocketSend {
    queue: Sender<Vec<u8>>
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {

    }
}

pub struct DomainSocketRecv {
    socket: Arc<Mutex<UnixStream>>,
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
        let path_gen = || format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        let server_send = path_gen();
        let client_send = path_gen();
        let server_recv = path_gen();
        let client_recv = path_gen();
        (
            serde_cbor::to_vec(&(true, &server_send, &server_recv)).unwrap(),
            serde_cbor::to_vec(&(false, &server_send, &server_recv)).unwrap(),
        )
    }

    type SendHalf = DomainSocketSend;
    type RecvHalf = DomainSocketRecv;

    fn new(data: Vec<u8>) -> Self {
        let (am_i_server, address_send, address_recv): (bool, String, String) = serde_cbor::from_slice(&data).unwrap();

        let connect = |is_server: bool, address: String| {
            if am_i_server {
                let listener = UnixListener::bind(&address).unwrap();
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
                        if let Ok(stream) = UnixStream::connect(&address) {
                            return stream
                        }
                        std::thread::sleep(std::time::Duration::from_millis(10));
                    }
                    panic!("Failed to establish domain socket within a timeout")
                })()
            }
        };

        // We use spinning for the connection establishment
        let mut stream_send = connect(am_i_server, address_send); 

        let mut stream_recv = connect(am_i_server, address_recv); 


        let mut poll1 = Poll::new().unwrap();
        poll1.registry()
        .register(&mut stream_send, POLL_TOKEN, Interest::WRITABLE).unwrap();

        let mut poll2 = Poll::new().unwrap();
        poll2.registry()
        .register(&mut stream_recv, POLL_TOKEN, Interest::READABLE).unwrap();


        DomainSocket {
            send: DomainSocketSend {
                socket: Arc::new(Mutex::new(stream_send)),
                poll: Mutex::new(poll1),
                _prevent_sync: PhantomData
            },
            recv: DomainSocketRecv {
                socket: Arc::new(Mutex::new(stream_recv)),
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

    println!("HAHAHAHA");

    let huge_data = {
        let mut v = Vec::new();
        for i in 0..300 {
            v.push((i % 255) as u8)
        }
        v
    };

    s2.send(&huge_data);

    let r = s1.recv(None).unwrap();

    s1.send(&huge_data);
    let r = s2.recv(None).unwrap();
}
