use crate::net::packet::{PacketHeader, PacketType, Reliability, Control};
use crate::serialize::Serialize;
use anyhow::Result;
use std::io::{Write, Cursor};
use tokio::net::{ToSocketAddrs, UdpSocket};
use crate::net::serverbound::ServerBound;
use crate::net::clientbound::ClientBound;

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

        // Initiate connection with an empty reliable packet
        conn.send(&[], true).await?;

        Ok(conn)
    }

    pub fn process_control(&mut self, control: &Control) {
        match control {
            Control::Ack { seqnum } => {},
            Control::SetPeerId { peer_id } => {
                println!("Setting peer_id = {}", peer_id);
                self.peer_id = *peer_id;
            }
            Control::Ping => {}
            Control::Disco => {}
        }
    }

    pub async fn receive_packet(&mut self) -> Result<(PacketHeader, Option<ClientBound>)> {
        let mut buf = [0; 1500];
        self.socket.recv(&mut buf).await?;

        let mut buf = Cursor::new(buf);

        let packet_header = PacketHeader::deserialize(&mut buf)?;

        let reliability = packet_header.reliability.clone();

        let data = match &packet_header.ty {
            PacketType::Control(_) => {
                Ok((packet_header, None))
            },
            PacketType::Original => {
                let clientbound = ClientBound::deserialize(&mut buf)?;
                Ok((packet_header, Some(clientbound)))
            },
        };

        println!("RECV {:?}", data);

        match reliability {
            Reliability::Reliable { seqnum } => {
                self.send_ack(seqnum).await?;
            }
            _ => (),
        }

        data
    }

    pub async fn send_packet(&mut self, packet: ServerBound, reliable: bool) -> Result<()> {
        println!("SEND {:?}", packet);
        let mut data = Vec::new();
        packet.serialize(&mut data)?;
        self.send(&data, reliable).await?;
        Ok(())
    }

    async fn send_ack(&mut self, seqnum: u16) -> Result<()> {
        let control = Control::Ack {
            seqnum,
        };

        let packet_header = PacketHeader {
            peer_id: self.peer_id,
            channel: 0,
            reliability: Reliability::Unreliable,
            ty: PacketType::Control(control),
        };

        println!("ACK  {:?}", packet_header);

        let mut buf = Vec::new();
        packet_header.serialize(&mut buf)?;

        self.socket.send(&buf).await?;
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
