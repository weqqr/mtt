use anyhow::Result;
use std::net::{UdpSocket, ToSocketAddrs};

pub struct Connection {
    socket: UdpSocket,
}

impl Connection {
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.connect(address)?;

        Ok(Self {
            socket,
        })
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.socket.send(bytes)?;
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut connection = Connection::new("127.0.0.1:30000")?;

    connection.send_bytes(b"")?;

    println!("Hello, world!");

    Ok(())
}
