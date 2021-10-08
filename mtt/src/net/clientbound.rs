use mtt_macros::packet;
use crate::serialize::RawBytes16;

#[packet]
#[derive(Debug, Clone)]
pub enum ClientBound {
    #[id = 0x0002]
    Hello {
        serialization_version: u8,
        compression_mode: u16,
        protocol_version: u16,
        supported_auth_modes: u32,
        legacy_player_name: String,
    },

    #[id = 0x0029]
    TimeOfDay { time: u16, time_speed: f32 },

    #[id = 0x0060]
    SrpBytesSB {
        s: RawBytes16,
        b: RawBytes16,
    }
}
