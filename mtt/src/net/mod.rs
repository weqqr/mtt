use crate::net::packet::{PacketHeader, PacketType, Reliability};
use crate::serialize::Serialize;
use anyhow::Result;
use std::io::Write;
use std::net::{ToSocketAddrs, UdpSocket};

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
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.connect(address)?;

        Ok(Self {
            socket,
            seqnum: 0xFFDC,
            peer_id: 0,
        })
    }

    pub fn send_payload(&mut self, payload: &[u8], reliable: bool) -> Result<()> {
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

        self.socket.send(&data)?;

        Ok(())
    }
}
