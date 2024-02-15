use std::io::Read;
use std::str::FromStr;
use roxmltree::{Document, Node};
use crate::{Color, Error, Gid, Layer, Orientation, Properties, Result, Tileset};


/// A tiled map parsed from a map file.
#[derive(Debug)]
pub struct Map {
    version: String,
    class: String,
    orientation: Orientation,
    render_order: RenderOrder,
    width: u32, 
    height: u32,
    tile_width: u32,
    tile_height: u32,
    hex_side_length: Option<i32>,
    stagger_axis: Option<StaggerAxis>,
    stagger_index: Option<StaggerIndex>,
    parallax_origin_x: f32,
    parallax_origin_y: f32,
    background_color: Color,
    tileset_entries: Vec<TilesetEntry>,
    infinite: bool,
    layers: Vec<Layer>,
    properties: Properties,
}

impl Default for Map {
    fn default() -> Self {
        Self {
            version: Default::default(),
            class: Default::default(),
            orientation: Default::default(),
            render_order: Default::default(),
            width: Default::default(),
            height: Default::default(),
            tile_width: Default::default(),
            tile_height: Default::default(),
            hex_side_length: Default::default(),
            stagger_axis: Default::default(),
            stagger_index: Default::default(),
            parallax_origin_x: Default::default(),
            parallax_origin_y: Default::default(),
            background_color: Color::TRANSPARENT,
            tileset_entries: Default::default(),
            infinite: Default::default(),
            layers: Default::default(),
            properties: Default::default(),
        }
    }
}

impl Map {
    pub fn version(&self) -> &str { &self.version }
    pub fn class(&self) -> &str { &self.class }
    pub fn orientation(&self) -> Orientation { self.orientation }
    pub fn render_order(&self) -> RenderOrder { self.render_order }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn tile_width(&self) -> u32 { self.tile_width }
    pub fn tile_height(&self) -> u32 { self.tile_height }
    pub fn hex_side_length(&self) -> Option<i32> { self.hex_side_length }
    pub fn stagger_axis(&self) -> Option<StaggerAxis> { self.stagger_axis }
    pub fn stagger_index(&self) -> Option<StaggerIndex> { self.stagger_index }
    pub fn parallax_origin_x(&self) -> f32 { self.parallax_origin_x }
    pub fn parallax_origin_y(&self) -> f32 { self.parallax_origin_y }
    pub fn background_color(&self) -> Color { self.background_color }
    pub fn tileset_entries(&self) -> &[TilesetEntry] { &self.tileset_entries }
    pub fn infinite(&self) -> bool { self.infinite }
    pub fn layers(&self) -> &[Layer] { &self.layers }
    pub fn properties(&self) -> &Properties{ &self.properties }

    /// Tileset index and local tile id of a [`Tile`](crate::Tile).
    pub fn tile_location_of(&self, gid: Gid) -> Option<(usize, u32)> {
        let gid = gid.value();
        for (tileset_idx, tileset) in self.tileset_entries.iter().rev().enumerate() {
            if gid >= tileset.first_gid {
                let tile_id = gid - tileset.first_gid;
                return Some((tileset_idx, tile_id));
            }
        }
        None
    }

    pub fn parse(mut read: impl Read) -> Result<Self> {
        let mut xml_str = String::new();
        read.read_to_string(&mut xml_str)?;
        Self::parse_str(&xml_str)
    }

    pub fn parse_str(xml_str: &str) -> Result<Self> {
        let mut map = Self::default();
        let map_doc = Document::parse(xml_str)?;
        let root = map_doc.root();
        for node in root.children() {
            let tag_name = node.tag_name().name();
            match tag_name {
                "map" => map.parse_node(node)?,
                _ => {},
            }
        }
        Ok(map)
    }

    /// Parses inner map element as a [`Map`].
    fn parse_node(&mut self, map_node: Node) -> Result<()> {

        // Attributes
        for attribute in map_node.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            match name {
                "version" => self.version = value.into(),
                "class" => self.class = value.into(),
                "orientation" => self.orientation = Orientation::parse(value)?,
                "renderorder" => self.render_order = RenderOrder::from_str(value)?,
                "width" => self.width = value.parse()?,
                "height" => self.height = value.parse()?,
                "tilewidth" => self.tile_width = value.parse()?,
                "tileheight" => self.tile_height = value.parse()?,
                "hexsidelength" => self.hex_side_length = Some(value.parse()?),
                "staggeraxis" => self.stagger_axis = Some(value.parse()?),
                "staggerindex" => self.stagger_index = Some(value.parse()?),
                "parallaxoriginx" => self.parallax_origin_x = value.parse()?,
                "parallaxoriginy" => self.parallax_origin_y = value.parse()?,
                "backgroundcolor" => self.background_color = value.parse()?,
                "infinite" => self.infinite = match value {
                    "0" => false,
                    "1" => true,
                    _ => return Err(Error::ParsingError),
                },
                _ => {}
            }
        }
    
        // Children
        for node in map_node.children() {
            match node.tag_name().name() {
                "tileset" => self.tileset_entries.push(TilesetEntry::parse(node)?),
                "properties" => self.properties = Properties::parse(node)?,
                // Note: According to spec, <tileset> elements always appear before <layer>, and <group> elements,
                // So the tilesets passed in are already complete.
                "layer" => {
                    let layer = Layer::parse_tile_layer(node, self.infinite)?;
                    self.layers.push(layer);
                },
                "group" => {
                    let layer = Layer::parse_group_layer(node, self.infinite)?;
                    self.layers.push(layer);
                },
                "imagelayer" => {
                    let layer = Layer::parse_image_layer(node)?;
                    self.layers.push(layer);
                },
                "objectgroup" => {
                    let layer = Layer::parse_object_group_layer(node)?;
                    self.layers.push(layer);
                },
                _ => {},
            }
        }

        Ok(())
    }
}

/// A single tileset stored in a [`Map`].
/// Either embeds the tileset, or references it in another file.
#[derive(Clone, Debug)]
pub struct TilesetEntry {
    first_gid: u32,
    kind: TilesetEntryKind,
}

impl TilesetEntry {

    pub fn first_gid(&self) -> u32 { self.first_gid }
    pub fn kind(&self) -> &TilesetEntryKind { &self.kind }

    fn parse(entry_node: Node) -> Result<Self> {
        let mut first_gid: u32 = 0;
        let mut source: Option<String> = None;
        for attr in entry_node.attributes() {
            match attr.name() {
                "firstgid" => first_gid = attr.value().parse()?,
                "source" => source = Some(attr.value().into()),
                _ => {}
            }
        }
        if let Some(source) = source {
            Ok(TilesetEntry::external(first_gid, source))
        }
        else {
            let mut tileset = Tileset::default();
            tileset.parse_node(entry_node)?;
            Ok(TilesetEntry::internal(first_gid, tileset))
        }
    }

    fn internal(first_gid: u32, tileset: Tileset) -> Self {
        Self {            
            first_gid,
            kind: TilesetEntryKind::Internal(tileset),
        }
    }

    fn external(first_gid: u32, source: String) -> Self {
        Self {            
            first_gid,
            kind: TilesetEntryKind::External(source),
        }
    }
}

/// Either embeds a tileset, or references an external one.
#[derive(Clone, Debug)]
pub enum TilesetEntryKind {
    Internal(Tileset),
    External(String),
}

/// The order in which tiles on tile layers are rendered.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum RenderOrder {
    #[default]
    RightDown,
    RightUp,
    LeftDown,
    LeftUp,
}

impl RenderOrder {
    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "right-down" => Ok(Self::RightDown),
            "right-up" => Ok(Self::RightUp),
            "left-down" => Ok(Self::LeftDown),
            "left-up" => Ok(Self::LeftUp),
            _ => Err(Error::ParsingError)
        }
    }
}

/// For staggered and hexagonal maps, determines which axis (X or Y) is staggered.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum StaggerAxis {
    X,
    #[default]
    Y,
    LeftDown,
    LeftUp,
}

impl FromStr for StaggerAxis {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "x" => Ok(Self::X),
            "y" => Ok(Self::Y),
            _ => Err(Error::ParsingError),
        }
    }
}

/// For staggered and hexagonal maps, determines whether the Even or Odd indexes along the staggered axis are shifted.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub enum StaggerIndex {
    Even,
    #[default]
    Odd,
}

impl FromStr for StaggerIndex {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "even" => Ok(Self::Even),
            "odd" => Ok(Self::Odd),
            _ => Err(Error::ParsingError),
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{ Gid, Map};

    #[test]
    fn test_finite() {
        let xml = include_str!("test_data/finite.tmx");
        let map = Map::parse_str(xml).unwrap();
        let layer = map.layers().iter().find(|layer| layer.name() == "below").unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();
        assert_eq!(Gid(2147484833), tile_layer.gid_at(0, 0));
        assert_eq!(Gid(98), tile_layer.gid_at(5, 2));
    }

    #[test]
    fn test_infinite() {
        let xml = include_str!("test_data/infinite.tmx");
        let map = Map::parse_str(xml).unwrap();
        let layer = map.layers().iter().find(|layer| layer.name() == "below").unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();
        assert_eq!(Gid(2147484833), tile_layer.gid_at(0, 0));
        assert_eq!(Gid(98), tile_layer.gid_at(5, 2));
    }

    #[test]
    fn test_hexagonal() {
        let xml = include_str!("test_data/hexagonal.tmx");
        let _map = Map::parse_str(xml).unwrap();
    }

    #[test]
    fn test_isometric() {
        let xml = include_str!("test_data/isometric.tmx");
        let _map = Map::parse_str(xml).unwrap();
    }

    #[test]
    fn test_isometric_staggered() {
        let xml = include_str!("test_data/isometric_staggered.tmx");
        let _map = Map::parse_str(xml).unwrap();
    }

    #[test]
    fn test_iter() {
        let xml = include_str!("test_data/infinite.tmx");
        let map = Map::parse_str(xml).unwrap();
        let layer = map.layers().iter().find(|layer| layer.name() == "below").unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();
        let mut gids = tile_layer.gids().non_null();

        let expected_x: i32 = -4;
        let expected_y: i32 = -2;
        let expected_gid = Gid(1);
        assert_eq!(Some((expected_x, expected_y, expected_gid)), gids.next());

        let expected_x: i32 = 0;
        let expected_y: i32 = 0;
        let expected_gid = Gid(2147484833);
        assert_eq!(Some((expected_x, expected_y, expected_gid)), gids.next());
    }

    #[test]
    fn test_image_layer() {
        let xml = include_str!("test_data/infinite.tmx");
        let map = Map::parse_str(xml).unwrap();
        let layer = map.layers().iter().find(|layer| layer.name() == "background").unwrap();
        let image_layer = layer.as_image_layer().unwrap();
        assert_eq!("images/pepe.png", image_layer.image().source());
    }

    #[test]
    fn test_object_layer() {
        let xml = include_str!("test_data/finite.tmx");
        let map = Map::parse_str(xml).unwrap();
        let layer = map.layers().iter().find(|layer| layer.name() == "objects").unwrap();
        let object_layer = layer.as_object_group_layer().unwrap();
        println!("{object_layer:#?}");
    }
}