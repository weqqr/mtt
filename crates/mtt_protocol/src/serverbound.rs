use mtt_macros::{packet, Serialize};
use mtt_serialize::{RawBytes16, RawBytesUnsized, Serialize};
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize)]
pub struct Handshake {}

#[derive(Debug, Clone, Serialize)]
pub struct Init {
    pub max_serialization_version: u8,
    pub supported_compression_modes: u16,
    pub min_protocol_version: u16,
    pub max_protocol_version: u16,
    pub player_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Init2 {
    pub language_code: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GotBlocks {
    pub count: u8,
    pub blocks: RawBytesUnsized,
}

#[derive(Debug, Clone)]
pub struct RequestMedia {
    pub media: Vec<String>,
}

impl Serialize for RequestMedia {
    fn serialize<W: Write>(&self, w: &mut W) -> anyhow::Result<()> {
        let len: u16 = self.media.len().try_into()?;
        len.serialize(w)?;

        for elem in &self.media {
            elem.serialize(w)?;
        }

        Ok(())
    }

    fn deserialize<R: Read>(_r: &mut R) -> anyhow::Result<Self> {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientReady {
    pub version_major: u8,
    pub version_minor: u8,
    pub version_patch: u8,
    pub reserved: u8,
    pub full_version: String,
    pub formspec_version: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct SrpBytesA {
    pub data: RawBytes16,
    pub based_on: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct SrpBytesM {
    pub data: RawBytes16,
}

#[packet]
#[derive(Debug, Clone)]
pub enum ServerBound {
    #[id = 0x00]
    Handshake(Handshake),

    #[id = 0x02]
    Init(Init),

    #[id = 0x11]
    Init2(Init2),

    #[id = 0x24]
    GotBlocks(GotBlocks),

    #[id = 0x40]
    RequestMedia(RequestMedia),

    #[id = 0x43]
    ClientReady(ClientReady),

    #[id = 0x51]
    SrpBytesA(SrpBytesA),

    #[id = 0x52]
    SrpBytesM(SrpBytesM),
}
