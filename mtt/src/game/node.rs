use crate::math::Aabb;
use crate::serialize::Serialize;
use anyhow::Result;
use bitflags::bitflags;
use mtt_macros::Serialize;
use std::collections::HashMap;
use std::io::{Read, Write};

#[derive(Debug, Clone)]
pub enum TileAnimation {
    None,
    VerticalFrames {
        aspect_w: u16,
        aspect_h: u16,
        length: f32,
    },
    Sheet {
        frames_w: u8,
        frames_h: u8,
        length: f32,
    },
}

impl Serialize for TileAnimation {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        unimplemented!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let ty = u8::deserialize(r)?;

        Ok(match ty {
            0 => TileAnimation::None,
            1 => TileAnimation::VerticalFrames {
                aspect_w: u16::deserialize(r)?,
                aspect_h: u16::deserialize(r)?,
                length: f32::deserialize(r)?,
            },
            2 => TileAnimation::Sheet {
                frames_w: u8::deserialize(r)?,
                frames_h: u8::deserialize(r)?,
                length: f32::deserialize(r)?,
            },
            _ => anyhow::bail!("invalid tile animation type: {}", ty),
        })
    }
}

bitflags! {
    struct TileFlags: u16 {
        const BACK_FACE_CULLING   = 1 << 0;
        const TILEABLE_HORIZONTAL = 1 << 1;
        const TILEABLE_VERTICAL   = 1 << 2;
        const HAS_COLOR           = 1 << 3;
        const HAS_SCALE           = 1 << 4;
        const HAS_ALIGNMENT       = 1 << 5;
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Argb {
    pub a: u8,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone)]
pub enum Alignment {
    None,
    World,
    UserDefined,
}

impl Serialize for Alignment {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        unimplemented!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let ty = u8::deserialize(r)?;
        Ok(match ty {
            0 => Alignment::None,
            1 => Alignment::World,
            2 => Alignment::UserDefined,
            _ => anyhow::bail!("invalid alignment type: {}", ty),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Tile {
    name: String,
    animation: TileAnimation,
    flags: TileFlags,
    color: Option<Rgb>,
    scale: u8,
    alignment: Alignment,
}

impl Serialize for Tile {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        unimplemented!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let version = u8::deserialize(r)?;
        anyhow::ensure!(version >= 6, "bad tile version");

        let name = String::deserialize(r)?;
        let animation = TileAnimation::deserialize(r)?;
        let flags = TileFlags::from_bits_truncate(u16::deserialize(r)?);

        let color = if flags.contains(TileFlags::HAS_COLOR) {
            Some(Rgb::deserialize(r)?)
        } else {
            None
        };

        let scale = if flags.contains(TileFlags::HAS_SCALE) {
            u8::deserialize(r)?
        } else {
            0
        };

        let alignment = if flags.contains(TileFlags::HAS_ALIGNMENT) {
            Alignment::deserialize(r)?
        } else {
            Alignment::None
        };

        Ok(Self {
            name,
            animation,
            flags,
            color,
            scale,
            alignment,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Lighting {
    light_propagates: bool,
    sunlight_propagates: bool,
    light_source: u8,
}

#[derive(Debug, Clone, Serialize)]
pub struct Interaction {
    walkable: bool,
    pointable: bool,
    diggable: bool,
    climbable: bool,
    buildable_to: bool,
    rightclickable: bool,
    damage_per_second: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Liquid {
    ty: u8,
    alternative_flowing: String,
    alternative_source: String,
    viscosity: u8,
    renewable: bool,
    range: u8,
    drowning: u8,
    floodable: bool,
}

#[derive(Debug, Clone)]
pub struct Boxes {
    boxes: Vec<Aabb>,
}

impl Serialize for Boxes {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        unimplemented!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        Ok(Self {
            boxes: Vec::<Aabb>::deserialize(r)?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct NodeBoxConnectors {
    top: Boxes,
    bottom: Boxes,
    front: Boxes,
    left: Boxes,
    back: Boxes,
    right: Boxes,
}

#[derive(Debug, Clone)]
pub enum NodeBox {
    Regular,
    Leveled(Boxes),
    Fixed(Boxes),
    WallMounted {
        top: Aabb,
        bottom: Aabb,
        side: Aabb,
    },
    Connected {
        connected: Boxes,
        connectors: NodeBoxConnectors,
        disconnected_connectors: NodeBoxConnectors,
        disconnected: Boxes,
        disconnected_sides: Boxes,
    },
}

impl Serialize for NodeBox {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let version = u8::deserialize(r)?;
        anyhow::ensure!(version >= 6, "bad nodebox version");
        let ty = u8::deserialize(r)?;
        Ok(match ty {
            0 => NodeBox::Regular,
            1 => NodeBox::Fixed(Boxes::deserialize(r)?),
            2 => NodeBox::WallMounted {
                top: Aabb::deserialize(r)?,
                bottom: Aabb::deserialize(r)?,
                side: Aabb::deserialize(r)?,
            },
            3 => NodeBox::Leveled(Boxes::deserialize(r)?),
            4 => NodeBox::Connected {
                connected: Boxes::deserialize(r)?,
                connectors: NodeBoxConnectors::deserialize(r)?,
                disconnected_connectors: NodeBoxConnectors::deserialize(r)?,
                disconnected: Boxes::deserialize(r)?,
                disconnected_sides: Boxes::deserialize(r)?,
            },
            _ => anyhow::bail!("unknown nodebox type: {}", ty),
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Sound {
    name: String,
    gain: f32,
    pitch: f32,
    fade: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct Sounds {
    footstep: Sound,
    dig: Sound,
    dug: Sound,
}

#[derive(Debug, Clone)]
pub enum DrawType {
    Normal,
    AirLike,
    Liquid,
    FlowingLiquid,
    GlassLike,
    AllFaces,
    AllFacesOptional,
    TorchLike,
    SignLike,
    PlantLike,
    FenceLike,
    RailLike,
    NodeBox,
    GlassLikeFramed,
    FireLike,
    GlassLikeFramedOptional,
    Mesh,
    PlantLikeRooted,
}

impl Serialize for DrawType {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        todo!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        let ty = u8::deserialize(r)?;
        Ok(match ty {
            0 => DrawType::Normal,
            1 => DrawType::AirLike,
            2 => DrawType::Liquid,
            3 => DrawType::FlowingLiquid,
            4 => DrawType::GlassLike,
            5 => DrawType::AllFaces,
            6 => DrawType::AllFacesOptional,
            7 => DrawType::TorchLike,
            8 => DrawType::SignLike,
            9 => DrawType::PlantLike,
            10 => DrawType::FenceLike,
            11 => DrawType::RailLike,
            12 => DrawType::NodeBox,
            13 => DrawType::GlassLikeFramed,
            14 => DrawType::FireLike,
            15 => DrawType::GlassLikeFramedOptional,
            16 => DrawType::Mesh,
            17 => DrawType::PlantLikeRooted,
            _ => anyhow::bail!("unknown DrawType: {}", ty),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Node {
    name: String,
    groups: HashMap<String, i16>,
    param_type1: u8,
    param_type2: u8,
    draw_type: DrawType,
    mesh: Option<String>,
    visual_scale: f32,
    tiles: Vec<Tile>,
    tiles_overlay: Vec<Tile>,
    tiles_special: Vec<Tile>,
    color: Rgb,
    palette_name: String,
    waving: u8,
    connect_sides: u8,
    connects_to: Vec<u16>,
    post_effect_color: Argb,
    leveled: u8,
    lighting: Lighting,
    is_ground_content: bool,
    interaction: Interaction,
    liquid: Liquid,
    node_box: NodeBox,
    selection_box: NodeBox,
    collision_box: NodeBox,
    sounds: Sounds,
}

impl Serialize for Node {
    fn serialize<W: Write>(&self, _w: &mut W) -> Result<()> {
        unimplemented!()
    }

    fn deserialize<R: Read>(r: &mut R) -> Result<Self> {
        // length of serialized NodeDef
        // TODO: Reader should be limited to this size
        let _ = u16::deserialize(r)?;

        let version = u8::deserialize(r)?;
        anyhow::ensure!(version >= 13);

        let name = String::deserialize(r)?;

        let groups_count = u16::deserialize(r)?;
        let mut groups = HashMap::new();
        for _ in 0..groups_count {
            let name = String::deserialize(r)?;
            let value = i16::deserialize(r)?;
            groups.insert(name, value);
        }

        let param_type1 = u8::deserialize(r)?;
        let param_type2 = u8::deserialize(r)?;

        let draw_type = DrawType::deserialize(r)?;
        let mesh = String::deserialize(r)?;
        let mesh = mesh.is_empty().then(|| mesh);

        let visual_scale = f32::deserialize(r)?;

        let tile_count = u8::deserialize(r)?;
        anyhow::ensure!(tile_count == 6);

        let mut tiles = Vec::new();
        for _ in 0..tile_count {
            let tile = Tile::deserialize(r)?;
            tiles.push(tile);
        }

        let mut tiles_overlay = Vec::new();
        for _ in 0..tile_count {
            tiles_overlay.push(Tile::deserialize(r)?);
        }

        let special_tile_count = u8::deserialize(r)?;
        anyhow::ensure!(special_tile_count == 6);

        let mut tiles_special = Vec::new();
        for _ in 0..special_tile_count {
            tiles_special.push(Tile::deserialize(r)?);
        }

        let _alpha = u8::deserialize(r)?;

        let color = Rgb::deserialize(r)?;

        let palette_name = String::deserialize(r)?;
        let waving = u8::deserialize(r)?;
        let connect_sides = u8::deserialize(r)?;
        let connects_to_count = u16::deserialize(r)?;
        let mut connects_to = Vec::new();
        for _ in 0..connects_to_count {
            connects_to.push(u16::deserialize(r)?);
        }
        let post_effect_color = Argb::deserialize(r)?;
        let leveled = u8::deserialize(r)?;
        let lighting = Lighting::deserialize(r)?;
        let is_ground_content = bool::deserialize(r)?;
        let interaction = Interaction::deserialize(r)?;
        let liquid = Liquid::deserialize(r)?;
        let node_box = NodeBox::deserialize(r)?;
        let selection_box = NodeBox::deserialize(r)?;
        let collision_box = NodeBox::deserialize(r)?;
        let sounds = Sounds::deserialize(r)?;

        let _ = u8::deserialize(r)?;
        let _ = u8::deserialize(r)?;

        // TODO: new, optional attributes
        let _node_dig_prediction = String::deserialize(r)?;
        let _ = u8::deserialize(r)?;
        let _ = u8::deserialize(r)?;
        let _ = u8::deserialize(r)?;
        let _ = u8::deserialize(r)?;

        Ok(Self {
            name,
            groups,
            param_type1,
            param_type2,
            draw_type,
            mesh,
            visual_scale,
            tiles,
            tiles_overlay,
            tiles_special,
            color,
            palette_name,
            waving,
            connect_sides,
            connects_to,
            post_effect_color,
            leveled,
            lighting,
            is_ground_content,
            interaction,
            liquid,
            node_box,
            selection_box,
            collision_box,
            sounds,
        })
    }
}
