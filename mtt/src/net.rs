use crate::serialize::Serialize;
use anyhow::{bail, ensure, Result};
use std::io::{Read, Write};
use std::net::{ToSocketAddrs, UdpSocket};

const PROTOCOL_ID: u32 = 0x4F457403;

#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error("protocol ID mismatch (got {0:08X})")]
    IdMismatch(u32),

    #[error("unknown packet type: {0}")]
    UnknownPacketType(u8),

    #[error("unknown control type: {0}")]
    UnknownControlType(u8),
}

pub enum Control {
    Ack { seqnum: u16 },
    SetPeerId { peer_id: u16 },
    Ping,
    Disco,
}

impl Serialize for Control {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        match self {
            Control::Ack { seqnum } => {
                0u8.serialize(w)?;
                seqnum.serialize(w)?;
            }
            Control::SetPeerId { peer_id } => {
                1u8.serialize(w)?;
                peer_id.serialize(w)?;
            }
            Control::Ping => 2u8.serialize(w)?,
            Control::Disco => 3u8.serialize(w)?,
        }

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let ty = u8::deserialize(r)?;

        Ok(match ty {
            0 => Control::Ack {
                seqnum: u16::deserialize(r)?,
            },
            1 => Control::SetPeerId {
                peer_id: u16::deserialize(r)?,
            },
            2 => Control::Ping,
            3 => Control::Disco,
            _ => bail!(ProtocolError::UnknownControlType(ty)),
        })
    }
}

pub enum PacketType {
    Control(Control),
    Original,
}

impl Serialize for PacketType {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        match self {
            PacketType::Control(control) => {
                0u8.serialize(w)?;
                control.serialize(w)?;
            }
            PacketType::Original => 1u8.serialize(w)?,
        }

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let ty = u8::deserialize(r)?;

        Ok(match ty {
            0 => PacketType::Control(Control::deserialize(r)?),
            1 => PacketType::Original,
            _ => bail!(ProtocolError::UnknownPacketType(ty)),
        })
    }
}

pub struct PacketHeader {
    peer_id: u16,
    channel: u8,
    ty: PacketType,
}

impl Serialize for PacketHeader {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        PROTOCOL_ID.serialize(w)?;

        self.peer_id.serialize(w)?;
        self.channel.serialize(w)?;
        self.ty.serialize(w)?;

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let protocol_id = u32::deserialize(r)?;

        ensure!(protocol_id == PROTOCOL_ID, ProtocolError::IdMismatch(protocol_id));

        let peer_id = u16::deserialize(r)?;
        let channel = u8::deserialize(r)?;
        let ty = PacketType::deserialize(r)?;

        Ok(Self { peer_id, channel, ty })
    }
}

pub struct Connection {
    socket: UdpSocket,
}

impl Connection {
    pub fn new<A: ToSocketAddrs>(address: A) -> Result<Self> {
        let socket = UdpSocket::bind("0.0.0.0:0")?;

        socket.connect(address)?;

        Ok(Self { socket })
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        self.socket.send(bytes)?;
        Ok(())
    }
}
