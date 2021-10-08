use mtt_macros::packet;
use crate::serialize::RawBytes16;

#[packet]
#[derive(Debug, Clone)]
pub enum ServerBound {
    #[id = 0x0000]
    Handshake {},

    #[id = 0x0002]
    Init {
        max_serialization_version: u8,
        supported_compression_modes: u16,
        min_protocol_version: u16,
        max_protocol_version: u16,
        player_name: String,
    },

    #[id = 0x0051]
    SrpBytesA {
        data: RawBytes16,
        based_on: u8,
    },

    #[id = 0x0052]
    SrpBytesM {
        data: RawBytes16,
    }
}
