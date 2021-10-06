use mtt_macros::packet;

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
}
