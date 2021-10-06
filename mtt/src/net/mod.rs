use crate::net::packet::{PacketHeader, PacketType, Reliability};
use crate::serialize::Serialize;
use anyhow::Result;
use std::io::{Write, Cursor};
use tokio::net::{ToSocketAddrs, UdpSocket};
use crate::net::serverbound::ServerBound;
use crate::net::clientbound::ClientBound;
use tokio::net::windows::named_pipe::PipeEnd::Client;

pub mod clientbound;
pub mod packet;
pub mod serverbound;

const SPLIT_THRESHOLD: usize = 400;

pub struct Connection {
    socket: UdpSocket,
    peer_id: u16,
    seqnum: u16,
}

impl Connection {
    pub async fn connect<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;

        socket.connect(address).await?;

        let mut conn = Self {
            socket,
            seqnum: 0xFFDC,
            peer_id: 0,
        };

        conn.perform_initial_exchange().await?;

        Ok(conn)
    }

    async fn perform_initial_exchange(&mut self) -> Result<()> {
        self.send_packet(ServerBound::Hello {
            test: 123,
        }, true).await?;

        let packet = self.receive_packet().await?;
        println!("{:?}", packet);
        
        Ok(())
    }

    pub async fn receive_packet(&mut self) -> Result<ClientBound> {
        let mut buf = [0; 1500];
        self.socket.recv(&mut buf).await?;
        let mut buf = Cursor::new(buf);

        let packet = PacketHeader::deserialize(&mut buf)?;
        let clientbound = ClientBound::deserialize(&mut buf)?;

        Ok(clientbound)
    }

    pub async fn send_packet(&mut self, packet: ServerBound, reliable: bool) -> Result<()> {
        let mut data = Vec::new();
        packet.serialize(&mut data)?;
        self.send(&data, reliable).await?;
        Ok(())
    }

    pub async fn send(&mut self, payload: &[u8], reliable: bool) -> Result<()> {
        if payload.len() > SPLIT_THRESHOLD {
            todo!("split packets")
        }

        let reliability = if reliable {
            let reliability = Reliability::Reliable { seqnum: self.seqnum };
            self.seqnum = self.seqnum.wrapping_add(1);
            reliability
        } else {
            Reliability::Unreliable
        };

        let packet_header = PacketHeader {
            peer_id: self.peer_id,
            channel: 0,
            reliability,
            ty: PacketType::Original,
        };

        let mut data = Vec::new();
        packet_header.serialize(&mut data)?;
        data.write(payload)?;

        self.socket.send(&data).await?;

        Ok(())
    }
}
