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
        let encoded_message: Vec<u8> =
            bincode::serialize(message).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        let len = u64::to_le_bytes(encoded_message.len() as u64);
        self.0.write_all(&len)?;
        self.0.write_all(&encoded_message)?;
        Ok(())
    }

    pub fn recv(&mut self) -> Result<Message, io::Error> {
        let mut len_buf = [0u8; 8];
        self.0.read_exact(&mut len_buf)?;
        let len = u64::from_le_bytes(len_buf) as usize;
        let mut buf = vec![0; len];
        self.0.read_exact(&mut buf)?;
        let message: Message =
            bincode::deserialize(&buf).map_err(|e| io::Error::new(ErrorKind::InvalidData, e))?;
        Ok(message)
    }
}

pub struct MessageServer {
    listener_thread: Option<JoinHandle<()>>,
    thread_kill: Sender<()>,
}

impl MessageServer {
    pub fn start<T>(event_sender: Sender<T>, port: u16) -> Self
    where
        T: From<(IpAddr, NetworkEvent)> + Send + 'static,
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
        T: From<(IpAddr, NetworkEvent)> + Send + 'static,
    {
        println!("Message server awaiting connections");
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
                    let transport = MessageTransporter::new(stream);
                    {
                        let transport = transport.try_clone().unwrap();
                        event_sender
                            .send(T::from((socket.ip(), NetworkEvent::Connect(transport))))
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
        mut transport: MessageTransporter,
        socket: SocketAddr,
    ) where
        T: From<(IpAddr, NetworkEvent)> + Send + 'static,
    {
        println!("New connection received from {}", socket);
        let src_addr = socket.ip();
        let send_event = |event: NetworkEvent| event_sender.send(T::from((src_addr, event)));
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
        println!("Message server stopping");
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
        T: From<NetworkEvent> + Send + 'static,
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
        T: From<NetworkEvent> + Send + 'static,
    {
        println!("Starting message client");
        while deathswitch.try_recv().is_err() {
            let _: Result<_, io::Error> = try {
                let stream = TcpStream::connect(socket)?;
                println!("Connected to server");
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

impl Drop for MessageClient {
    fn drop(&mut self) {
        self.thread_kill.send(()).unwrap();
        self.connection_thread.take().unwrap().join().unwrap();
    }
}
