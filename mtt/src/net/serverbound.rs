use crate::serialize::{RawBytes16, RawBytesUnsized};
use mtt_macros::{packet, Serialize};

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

    #[id = 0x43]
    ClientReady(ClientReady),

    #[id = 0x51]
    SrpBytesA(SrpBytesA),

    #[id = 0x52]
    SrpBytesM(SrpBytesM),
}
