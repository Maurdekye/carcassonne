use std::{
    io::{self, ErrorKind, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    Ping,
    Pong,
    ChooseColor(usize),
}

pub enum NetworkEvent {
    Connect(MessageTransporter),
    Message(Message),
    Disconnect,
}

pub struct MessageTransporter(TcpStream);

impl MessageTransporter {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub fn try_clone(&self) -> Result<Self, io::Error> {
        Ok(MessageTransporter(self.0.try_clone()?))
    }

    pub fn send(&mut self, message: &Message) -> Result<(), io::Error> {
        let mut encoded_message: Vec<u8> = bincode::serialize(message)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        let mut len = u64::to_le_bytes(encoded_message.len() as u64);
        self.0.write_all(&mut len)?;
        self.0.write_all(&mut encoded_message)?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Message, io::Error> {
        let mut len_buf = [0u8; 8];
        self.0.read_exact(&mut len_buf)?;
        let len = u64::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0; len];
        self.0.read_exact(&mut buf)?;
        let message: Message = bincode::deserialize(&buf)
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        Ok(message)
    }
}

pub struct MessageServer<T> {
    event_sender: Sender<T>,
    listener_thread: Option<JoinHandle<()>>,
    thread_kill: Sender<()>,
}

impl<T> MessageServer<T>
where
    T: From<(IpAddr, NetworkEvent)> + Send + 'static,
{
    pub fn start(event_sender: Sender<T>, port: u16) -> Self {
        let (thread_kill, deathswitch) = channel();
        let listener_thread = {
            let event_sender = event_sender.clone();
            let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), port));
            Some(thread::spawn(move || {
                Self::listener_thread(event_sender, deathswitch, addr)
            }))
        };
        MessageServer {
            event_sender,
            listener_thread,
            thread_kill,
        }
    }

    fn listener_thread(event_sender: Sender<T>, deathswitch: Receiver<()>, addr: SocketAddr) {
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        while !deathswitch.try_recv().is_ok() {
            match listener.accept() {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Ok((stream, socket)) => {
                    let event_sender = event_sender.clone();
                    let transport = MessageTransporter::new(stream);
                    {
                        let transport = transport.try_clone().unwrap();
                        event_sender
                            .send(T::from((socket.ip(), NetworkEvent::Connect(transport))))
                            .unwrap();
                    }
                    thread::spawn(move || {
                        Self::connection_thread(event_sender, transport, socket)
                    });
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn connection_thread(
        event_sender: Sender<T>,
        mut transport: MessageTransporter,
        socket: SocketAddr,
    ) {
        let src_addr = socket.ip();
        let send_event = |event: NetworkEvent| event_sender.send(T::from((src_addr, event)));
        let result: Result<(), io::Error> = try {
            loop {
                (send_event)(NetworkEvent::Message(transport.recv()?)).unwrap();
            }
        };
        (send_event)(NetworkEvent::Disconnect).unwrap();
        result.unwrap()
    }
}

impl<T> Drop for MessageServer<T> {
    fn drop(&mut self) {
        self.thread_kill.send(()).unwrap();
        self.listener_thread.take().unwrap().join().unwrap();
    }
}

pub struct MessageClient<T> {
    event_sender: Sender<T>,
    connection_thread: Option<JoinHandle<()>>,
    thread_kill: Sender<()>,
}

impl<T> MessageClient<T>
where
    T: From<NetworkEvent> + Send + 'static,
{
    pub fn start(event_sender: Sender<T>, socket: SocketAddr) -> Self {
        let (thread_kill, deathswitch) = channel();
        let connection_thread = {
            let event_sender = event_sender.clone();
            Some(thread::spawn(move || {
                Self::connection_thread(event_sender, socket, deathswitch)
            }))
        };
        MessageClient {
            event_sender,
            connection_thread,
            thread_kill,
        }
    }

    fn connection_thread(
        event_sender: Sender<T>,
        socket: SocketAddr,
        deathswitch: Receiver<()>,
    ) {
        if !deathswitch.try_recv().is_ok() {
            let _: Result<_, io::Error> = try {
                let stream = TcpStream::connect(socket)?;
                let _: Result<_, io::Error> = try {
                    let mut transport = MessageTransporter::new(stream);
                    {
                        let transport = transport.try_clone().unwrap();
                        event_sender
                            .send(NetworkEvent::Connect(transport).into())
                            .unwrap();
                    }
                    loop {
                        event_sender
                            .send(NetworkEvent::Message(transport.recv()?).into())
                            .unwrap();
                    }
                };
                event_sender.send(NetworkEvent::Disconnect.into()).unwrap();
            };
        }
    }
}

impl<T> Drop for MessageClient<T> {
    fn drop(&mut self) {
        self.thread_kill.send(()).unwrap();
        self.connection_thread.take().unwrap().join().unwrap();
    }
}
