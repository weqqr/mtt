use glam::{Vec3, I16Vec3};
use mtt_core::world::Block;
use mtt_macros::{packet, Serialize};
use mtt_serialize::{RawBytes16, RawBytes32, Serialize, StringSerializeExt};
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Clone, Serialize)]
pub struct Hello {
    pub serialization_version: u8,
    pub compression_mode: u16,
    pub protocol_version: u16,
    pub supported_auth_modes: u32,
    pub legacy_player_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct AuthAccept {
    pub player_position: Vec3,
    pub seed: u64,
    pub send_interval: f32,
    pub supported_sudo_auth_methods: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct BlockData {
    pub position: I16Vec3,
    pub block: Block,
}

#[derive(Debug, Clone, Serialize)]
pub struct Inventory {}

#[derive(Debug, Clone, Serialize)]
pub struct TimeOfDay {
    pub time: u16,
    pub time_speed: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct CsmRestrictionFlags {
    pub flags: u64,
    pub range: u32,
}

#[derive(Debug, Clone)]
pub struct ChatMessage {
    pub version: u8,
    pub ty: u8,
    pub sender: String,
    pub message: String,
    pub time: u64,
}

impl Serialize for ChatMessage {
    fn serialize<W: Write>(&self, _w: &mut W) -> anyhow::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> anyhow::Result<Self> {
        Ok(ChatMessage {
            version: u8::deserialize(r)?,
            ty: u8::deserialize(r)?,
            sender: String::deserialize_utf16(r)?,
            message: String::deserialize_utf16(r)?,
            time: u64::deserialize(r)?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveObjectRemoveAdd {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct ActiveObjectMessages {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct Hp {
    pub hp: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct MovePlayer {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

#[derive(Debug, Clone)]
pub struct Media {
    pub bunch_count: u16,
    pub bunch_id: u16,
    pub files: Vec<(String, Vec<u8>)>,
}

impl Serialize for Media {
    fn serialize<W: Write>(&self, _w: &mut W) -> anyhow::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> anyhow::Result<Self> {
        let bunch_count = u16::deserialize(r)?;
        let bunch_id = u16::deserialize(r)?;
        let file_count = u32::deserialize(r)?;

        let mut files = Vec::new();
        for _ in 0..file_count {
            let name = String::deserialize(r)?;
            let data = RawBytes32::deserialize(r)?.0;

            files.push((name, data));
        }

        Ok(Self {
            bunch_count,
            bunch_id,
            files,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeDef {
    pub data: RawBytes32,
}

#[derive(Debug, Clone)]
pub struct AnnounceMedia {
    pub digests: HashMap<String, Vec<u8>>,
    pub content_servers: Vec<String>,
}

impl Serialize for AnnounceMedia {
    fn serialize<W: Write>(&self, _w: &mut W) -> anyhow::Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> anyhow::Result<Self> {
        let mut digests = HashMap::new();
        let count = u16::deserialize(r)?;
        for _ in 0..count {
            let name = String::deserialize(r)?;
            let digest = base64::decode(String::deserialize(r)?)?;
            digests.insert(name, digest);
        }

        let content_servers = String::deserialize(r)?.split(',').map(|s| s.to_owned()).collect();

        Ok(Self {
            digests,
            content_servers,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ItemDef {
    pub data: RawBytes32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Privileges {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct InventoryFormspec {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct DetachedInventory {
    pub name: String,
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct Movement {
    pub acceleration_default: f32,
    pub acceleration_air: f32,
    pub acceleration_fast: f32,
    pub speed_walk: f32,
    pub speed_crouch: f32,
    pub speed_fast: f32,
    pub speed_climb: f32,
    pub speed_jump: f32,
    pub liquid_fluidity: f32,
    pub liquid_fluidity_smooth: f32,
    pub liquid_sink: f32,
    pub gravity: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct HudAdd {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct HudChange {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct HudSetFlags {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct Breath {
    pub breath: u16,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdatePlayerList {
    // TODO
}

#[derive(Debug, Clone, Serialize)]
pub struct SrpBytesSB {
    pub s: RawBytes16,
    pub b: RawBytes16,
}

#[packet]
#[derive(Debug, Clone)]
pub enum ClientBound {
    #[id = 0x02]
    Hello(Hello),

    #[id = 0x03]
    AuthAccept(AuthAccept),

    #[id = 0x20]
    BlockData(BlockData),

    #[id = 0x27]
    Inventory(Inventory),

    #[id = 0x29]
    TimeOfDay(TimeOfDay),

    #[id = 0x2A]
    CsmRestrictionFlags(CsmRestrictionFlags),

    #[id = 0x2F]
    ChatMessage(ChatMessage),

    #[id = 0x31]
    ActiveObjectRemoveAdd(ActiveObjectRemoveAdd),

    #[id = 0x32]
    ActiveObjectMessages(ActiveObjectMessages),

    #[id = 0x33]
    Hp(Hp),

    #[id = 0x34]
    MovePlayer(MovePlayer),

    #[id = 0x38]
    Media(Media),

    #[id = 0x3A]
    NodeDef(NodeDef),

    #[id = 0x3C]
    AnnounceMedia(AnnounceMedia),

    #[id = 0x3D]
    ItemDef(ItemDef),

    #[id = 0x41]
    Privileges(Privileges),

    #[id = 0x42]
    InventoryFormspec(InventoryFormspec),

    #[id = 0x43]
    DetachedInventory(DetachedInventory),

    #[id = 0x45]
    Movement(Movement),

    #[id = 0x49]
    HudAdd(HudAdd),

    #[id = 0x4B]
    HudChange(HudChange),

    #[id = 0x4C]
    HudSetFlags(HudSetFlags),

    #[id = 0x4E]
    Breath(Breath),

    #[id = 0x56]
    UpdatePlayerList(UpdatePlayerList),

    #[id = 0x60]
    SrpBytesSB(SrpBytesSB),
}
