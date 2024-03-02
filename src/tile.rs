use roxmltree::Node;
use crate::{Image, ObjectGroupLayer, Properties, Result};


/// A tile belonging to a [`Tileset`](crate::Tileset).
#[derive(Clone, Default, Debug)]
pub struct Tile {
    typ: String,
    properties: Properties,
    image: Option<Image>,
    animation: Option<Animation>,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    objects: Option<ObjectGroupLayer>,
}

impl Tile {
    pub fn typ(&self) -> &str { &self.typ }
    pub fn properties(&self) -> &Properties { &self.properties }
    pub fn image(&self) -> Option<&Image> { self.image.as_ref() }
    pub fn x(&self) -> Option<u32> { self.x }
    pub fn y(&self) -> Option<u32> { self.y }
    pub fn width(&self) -> Option<u32> { self.width }
    pub fn height(&self) -> Option<u32> { self.height }
    pub fn animation(&self) -> Option<&Animation> { self.animation.as_ref() }
    pub fn objects(&self) -> Option<&ObjectGroupLayer> { self.objects.as_ref() }

    pub(crate) fn parse(tile_node: Node) -> Result<(u32, Tile)> {

        // Attributes
        let mut id = 0;
        let mut result = Self::default();
        for attr in tile_node.attributes() {
            match attr.name() {
                "id" => id = attr.value().parse()?,
                "x" => result.x = Some(attr.value().parse()?),
                "y" => result.y = Some(attr.value().parse()?),
                "width" => result.width = Some(attr.value().parse()?),
                "height" => result.height = Some(attr.value().parse()?),
                _ => {}
            }
        }

        // Children
        for child in tile_node.children() {
            match child.tag_name().name() {
                "properties" => result.properties = Properties::parse(child)?,
                "image" => result.image = Some(Image::parse(child)?),
                "animation" => result.animation = Some(Animation::parse(child)?),
                "objectgroup" => result.objects = Some(ObjectGroupLayer::parse(child)?),
                _ => {}
            }
        }

        Ok((id, result))
    }
}

/// Global id of a tile in a [`Map`](crate::Map).
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub struct Gid(pub u32);

impl Gid {
    pub const NULL: Self = Gid(0);

    const FLIP_MASK: u32                        = 0b00001111_11111111_11111111_11111111;
    pub const FLIPPED_HORIZONTALLY_FLAG: u32    = 0b10000000_00000000_00000000_00000000;
    pub const FLIPPED_VERTICALLY_FLAG: u32      = 0b01000000_00000000_00000000_00000000;
    pub const FLIPPED_DIAGONALLY_FLAG: u32      = 0b00100000_00000000_00000000_00000000;
    pub const ROTATED_HEXAGONAL_120_FLAG: u32   = 0b00010000_00000000_00000000_00000000;

    /// GID as an integer, with flip/rotation information stripped out.
    /// Use this when looking up tilesets.
    pub fn value(self) -> u32 { self.0 & Self::FLIP_MASK }

    pub fn is_flipped_horizontally(self) -> bool {
        self.0 & Self::FLIPPED_HORIZONTALLY_FLAG != 0
    }

    pub fn is_flipped_vertically(self) -> bool {
        self.0 & Self::FLIPPED_VERTICALLY_FLAG != 0
    }

    pub fn is_flipped_diagonally(self) -> bool {
        self.0 & Self::FLIPPED_DIAGONALLY_FLAG != 0
    }

    pub fn is_rotated_hex_120(self) -> bool {
        self.0 & Self::ROTATED_HEXAGONAL_120_FLAG != 0
    }
}

/// Animation frames of a [`Tile`].
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Animation(Vec<Frame>);
impl Animation {
    
    pub fn frames(&self) -> &[Frame] { &self.0 }

    pub(crate) fn parse(animation_node: Node) -> Result<Self> {
        let mut frames = Vec::new();
        for frame_node in animation_node.children().filter(|node| node.tag_name().name() == "frame") {
            let mut frame = Frame::default();
            for attr in frame_node.attributes() {
                match attr.name() {
                    "tileid" => frame.tile_id = attr.value().parse()?,
                    "duration" => frame.duration = attr.value().parse()?,
                    _ => {}
                }
            }
            frames.push(frame);
        }
        Ok(Self(frames))
    }
}

/// A frame in a tile [`Animation`].
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct Frame {
    pub tile_id: u32,
    pub duration: u32,
}