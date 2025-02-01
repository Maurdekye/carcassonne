use core::panic;
use std::{
    io::{self, ErrorKind, Read, Write},
    net::{IpAddr, Ipv4Addr, SocketAddr, SocketAddrV4, TcpListener, TcpStream},
    ops::{Deref, DerefMut},
    sync::mpsc::{channel, Receiver, Sender},
    thread::{self, JoinHandle},
    time::Duration,
};

use message::{client::ClientMessage, server::ServerMessage, Message};

pub mod message;

pub enum NetworkEvent<T, M> {
    Connect {
        transport: T,
        my_socket_addr: SocketAddr,
    },
    Message(M),
    Disconnect,
}

pub type ServerNetworkEvent = NetworkEvent<ServersideTransport, ClientMessage>;
pub type ClientNetworkEvent = NetworkEvent<ClientsideTransport, ServerMessage>;

pub struct MessageTransporter(TcpStream);

impl MessageTransporter {
    fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    fn try_clone(&self) -> Result<Self, io::Error> {
        Ok(MessageTransporter(self.0.try_clone()?))
    }

    fn send(&mut self, message: &Message) -> Result<(), io::Error> {
        let encoded_message: Vec<u8> =
            bincode::serialize(message).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        let len = u64::to_le_bytes(encoded_message.len() as u64);
        self.0.write_all(&len)?;
        self.0.write_all(&encoded_message)?;
        Ok(())
    }

    fn recv(&mut self) -> Result<Message, io::Error> {
        let mut len_buf = [0u8; 8];
        self.0.read_exact(&mut len_buf)?;
        let len = u64::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0; len];
        self.0.read_exact(&mut buf)?;
        let message: Message = bincode::deserialize(&buf[..])
            .map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        Ok(message)
    }

    pub fn shutdown(&mut self) -> Result<(), std::io::Error> {
        self.0.shutdown(std::net::Shutdown::Both)
    }
}

impl Deref for MessageTransporter {
    type Target = TcpStream;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ClientsideTransport(MessageTransporter);

impl ClientsideTransport {
    pub fn new(stream: TcpStream) -> Self {
        Self(MessageTransporter::new(stream))
    }

    pub fn try_clone(&self) -> Result<Self, io::Error> {
        Ok(Self(self.0.try_clone()?))
    }

    pub fn send(&mut self, message: ClientMessage) -> Result<(), io::Error> {
        self.0.send(&Message::Client(message))
    }

    pub fn blind_send(&mut self, message: ClientMessage) {
        let _ = self.send(message);
    }

    pub fn recv(&mut self) -> Result<ServerMessage, io::Error> {
        let Message::Server(message) = self.0.recv()? else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Received a clientside message from the server",
            ));
        };
        Ok(message)
    }
}

impl Deref for ClientsideTransport {
    type Target = MessageTransporter;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ClientsideTransport {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct ServersideTransport(MessageTransporter);

impl ServersideTransport {
    pub fn new(stream: TcpStream) -> Self {
        Self(MessageTransporter::new(stream))
    }

    pub fn try_clone(&self) -> Result<Self, io::Error> {
        Ok(Self(self.0.try_clone()?))
    }

    pub fn send(&mut self, message: ServerMessage) -> Result<(), io::Error> {
        self.0.send(&Message::Server(message))
    }

    pub fn blind_send(&mut self, message: ServerMessage) {
        let _ = self.send(message);
    }

    pub fn recv(&mut self) -> Result<ClientMessage, io::Error> {
        let Message::Client(message) = self.0.recv()? else {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "Received a serverside message from a client",
            ));
        };
        Ok(message)
    }
}

impl Deref for ServersideTransport {
    type Target = MessageTransporter;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ServersideTransport {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub struct MessageServer {
    listener_thread: Option<JoinHandle<()>>,
    thread_kill: Sender<()>,
}

impl MessageServer {
    pub fn start<T>(event_sender: Sender<T>, port: u16) -> Self
    where
        T: From<(IpAddr, ServerNetworkEvent)> + Send + 'static,
    {
        let (thread_kill, deathswitch) = channel();
        let listener_thread = {
            Some(thread::spawn(move || {
                Self::listener_thread(event_sender, deathswitch, port)
            }))
        };
        MessageServer {
            listener_thread,
            thread_kill,
        }
    }

    fn listener_thread<T>(event_sender: Sender<T>, deathswitch: Receiver<()>, port: u16)
    where
        T: From<(IpAddr, ServerNetworkEvent)> + Send + 'static,
    {
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
        let listener = TcpListener::bind(addr).unwrap();
        listener.set_nonblocking(true).unwrap();
        while deathswitch.try_recv().is_err() {
            match listener.accept() {
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(100));
                }
                Ok((stream, socket)) => {
                    let event_sender = event_sender.clone();
                    stream.set_nonblocking(false).unwrap();
                    let mut transport = ServersideTransport::new(stream);
                    let _ = transport.0.send(&Message::YourSocket(socket));
                    let Ok(Message::YourSocket(my_socket_addr)) = transport.0.recv() else {
                        eprintln!("Expected socket response from client");
                        continue;
                    };
                    {
                        let transport = transport.try_clone().unwrap();
                        event_sender
                            .send(T::from((
                                socket.ip(),
                                NetworkEvent::Connect {
                                    transport,
                                    my_socket_addr,
                                },
                            )))
                            .unwrap();
                    }
                    thread::spawn(move || Self::connection_thread(event_sender, transport, socket));
                }
                Err(e) => panic!("{e}"),
            }
        }
    }

    fn connection_thread<T>(
        event_sender: Sender<T>,
        mut transport: ServersideTransport,
        socket: SocketAddr,
    ) where
        T: From<(IpAddr, ServerNetworkEvent)> + Send + 'static,
    {
        let src_addr = socket.ip();
        let send_event = |event: ServerNetworkEvent| event_sender.send(T::from((src_addr, event)));
        let Err(err): Result<(), io::Error> = (try {
            loop {
                (send_event)(NetworkEvent::Message(transport.recv()?))
                    .map_err(|_| io::Error::new(ErrorKind::ConnectionAborted, "Channel closed"))?;
            }
        }) else {
            return;
        };
        println!("{err}");
        let _ = (send_event)(NetworkEvent::Disconnect);
    }
}

impl Drop for MessageServer {
    fn drop(&mut self) {
        self.thread_kill.send(()).unwrap();
        self.listener_thread.take().unwrap().join().unwrap();
    }
}

pub struct MessageClient {
    connection_thread: Option<JoinHandle<()>>,
    thread_kill: Sender<()>,
}

impl MessageClient {
    pub fn start<T>(event_sender: Sender<T>, socket: SocketAddr) -> Self
    where
        T: From<ClientNetworkEvent> + Send + 'static,
    {
        let (thread_kill, deathswitch) = channel();
        let connection_thread = Some(thread::spawn(move || {
            Self::connection_thread(event_sender, socket, deathswitch)
        }));
        MessageClient {
            connection_thread,
            thread_kill,
        }
    }

    fn connection_thread<T>(event_sender: Sender<T>, socket: SocketAddr, deathswitch: Receiver<()>)
    where
        T: From<ClientNetworkEvent> + Send + 'static,
    {
        while deathswitch.try_recv().is_err() {
            let _: Result<_, io::Error> = try {
                let stream = TcpStream::connect(socket)?;
                let Err(err): Result<_, io::Error> = (try {
                    let mut transport = ClientsideTransport::new(stream);
                    {
                        let mut transport = transport.try_clone().unwrap();
                        let Ok(Message::YourSocket(my_socket_addr)) = transport.0.recv() else {
                            panic!("Expected socket message from server");
                        };
                        let _ = transport.0.send(&Message::YourSocket(socket));
                        event_sender
                            .send(
                                NetworkEvent::Connect {
                                    transport,
                                    my_socket_addr,
                                }
                                .into(),
                            )
                            .unwrap();
                    }
                    loop {
                        event_sender
                            .send(NetworkEvent::Message(transport.recv()?).into())
                            .map_err(|_| {
                                io::Error::new(ErrorKind::ConnectionAborted, "Channel Closed")
                            })?;
                    }
                }) else {
                    return;
                };
                println!("{err}");
                let _ = event_sender.send(NetworkEvent::Disconnect.into());
            };
        }
    }
}

impl Drop for MessageClient {
    fn drop(&mut self) {
        self.thread_kill.send(()).unwrap();
        self.connection_thread.take().unwrap().join().unwrap();
    }
}
