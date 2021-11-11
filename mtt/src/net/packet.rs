use crate::serialize::Serialize;
use anyhow::{bail, ensure, Result};
use std::io::{Read, Write};

const PROTOCOL_ID: u32 = 0x4F457403;

#[derive(Debug)]
pub enum Control {
    Ack {
        seqnum: u16,
    },
    SetPeerId {
        peer_id: u16,
    },
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
            _ => bail!("unknown control type: {}", ty),
        })
    }
}

#[derive(Debug)]
pub struct Split {
    pub seqnum: u16,
    pub chunk_count: u16,
    pub chunk_number: u16,
}

impl Serialize for Split {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        self.seqnum.serialize(w)?;
        self.chunk_count.serialize(w)?;
        self.chunk_number.serialize(w)
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Self {
            seqnum: u16::deserialize(r)?,
            chunk_count: u16::deserialize(r)?,
            chunk_number: u16::deserialize(r)?,
        })
    }
}

#[derive(Debug)]
pub enum PacketType {
    Control(Control),
    Original,
    Split(Split),
}

#[derive(Debug, Clone)]
pub enum Reliability {
    Reliable {
        seqnum: u16,
    },
    Unreliable,
}

#[derive(Debug)]
pub struct PacketHeader {
    pub peer_id: u16,
    pub channel: u8,
    pub reliability: Reliability,
    pub ty: PacketType,
}

impl Serialize for PacketHeader {
    fn serialize<W: Write>(&self, w: &mut W) -> Result<()> {
        PROTOCOL_ID.serialize(w)?;

        self.peer_id.serialize(w)?;
        self.channel.serialize(w)?;

        if let Reliability::Reliable { seqnum } = self.reliability {
            3u8.serialize(w)?;
            seqnum.serialize(w)?;
        }

        match self.ty {
            PacketType::Control(ref control) => {
                0u8.serialize(w)?;
                control.serialize(w)?;
            }
            PacketType::Original => 1u8.serialize(w)?,
            PacketType::Split(ref split) => {
                2u8.serialize(w)?;
                split.serialize(w)?;
            }
        }

        Ok(())
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let protocol_id = u32::deserialize(r)?;

        ensure!(
            protocol_id == PROTOCOL_ID,
            "protocol ID mismatch (got {0:08X})",
            protocol_id
        );

        let peer_id = u16::deserialize(r)?;
        let channel = u8::deserialize(r)?;
        let mut ty = u8::deserialize(r)?;

        let reliability = if ty == 3 {
            let seqnum = u16::deserialize(r)?;
            ty = u8::deserialize(r)?;
            Reliability::Reliable { seqnum }
        } else {
            Reliability::Unreliable
        };

        let ty = match ty {
            0 => PacketType::Control(Control::deserialize(r)?),
            1 => PacketType::Original,
            2 => PacketType::Split(Split::deserialize(r)?),
            _ => bail!("unknown packet type: {}", ty),
        };

        Ok(Self {
            peer_id,
            channel,
            reliability,
            ty,
        })
    }
}
