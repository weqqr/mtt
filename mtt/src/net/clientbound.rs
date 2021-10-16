use crate::math::{Vector3, Vector3i16};
use crate::serialize::{RawBytes16, RawBytes32};
use crate::world::Block;
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

    #[id = 0x0003]
    AuthAccept {
        player_position: Vector3,
        seed: u64,
        send_interval: f32,
        supported_sudo_auth_methods: u32,
    },

    #[id = 0x0020]
    BlockData {
        position: Vector3i16,
        block: Block,
    },

    #[id = 0x0027]
    Inventory {
        // TODO
    },

    #[id = 0x0029]
    TimeOfDay {
        time: u16,
        time_speed: f32,
    },

    #[id = 0x002A]
    CsmRestrictionFlags {
        flags: u64,
        range: u32,
    },

    #[id = 0x002F]
    ChatMessage {
        // TODO
    },

    #[id = 0x0031]
    ActiveObjectRemoveAdd {
        // TODO
    },

    #[id = 0x0032]
    ActiveObjectMessages {
        // TODO
    },

    #[id = 0x0033]
    Hp {
        hp: u16,
    },

    #[id = 0x0034]
    MovePlayer {
        position: Vector3,
        pitch: f32,
        yaw: f32,
    },

    #[id = 0x003A]
    NodeDef {
        data: RawBytes32,
    },

    #[id = 0x003C]
    AnnounceMedia {
        // TODO
    },

    #[id = 0x003D]
    ItemDef {
        data: RawBytes32,
    },

    #[id = 0x0041]
    Privileges {
        // TODO
    },

    #[id = 0x0042]
    InventoryFormspec {
        // TODO
    },

    #[id = 0x0043]
    DetachedInventory {
        name: String,
        // TODO
    },

    #[id = 0x0045]
    Movement {
        acceleration_default: f32,
        acceleration_air: f32,
        acceleration_fast: f32,
        speed_walk: f32,
        speed_crouch: f32,
        speed_fast: f32,
        speed_climb: f32,
        speed_jump: f32,
        liquid_fluidity: f32,
        liquid_fluidity_smooth: f32,
        liquid_sink: f32,
        gravity: f32,
    },

    #[id = 0x0049]
    HudAdd {
        // TODO
    },

    #[id = 0x004B]
    HudChange {
        // TODO
    },

    #[id = 0x004C]
    HudSetFlags {
        // TODO
    },

    #[id = 0x004E]
    Breath {
        breath: u16,
    },

    #[id = 0x0056]
    UpdatePlayerList {
        // TODO
    },

    #[id = 0x0060]
    SrpBytesSB {
        s: RawBytes16,
        b: RawBytes16,
    },
}
