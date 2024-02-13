use std::io::Read;
use roxmltree::{Document, Node};
use crate::{Error, Layer, Orientation, Properties, Result, Tileset};


#[derive(Default, Debug)]
pub struct TiledMap {
    version: String,
    orientation: Orientation,
    render_order: RenderOrder,
    width: u32, 
    height: u32,
    tile_width: u32,
    tile_height: u32,
    tileset_entries: Vec<TilesetEntry>,
    infinite: bool,
    layers: Vec<Layer>,
    properties: Properties,
}

impl TiledMap {
    pub fn version(&self) -> &str { &self.version }
    pub fn orientation(&self) -> Orientation { self.orientation }
    pub fn render_order(&self) -> RenderOrder { self.render_order }
    pub fn width(&self) -> u32 { self.width }
    pub fn height(&self) -> u32 { self.height }
    pub fn tile_width(&self) -> u32 { self.tile_width }
    pub fn tile_height(&self) -> u32 { self.tile_height }
    pub fn tileset_entries(&self) -> &[TilesetEntry] { &self.tileset_entries }
    pub fn infinite(&self) -> bool { self.infinite }
    pub fn layers(&self) -> &[Layer] { &self.layers }
    pub fn properties(&self) -> &Properties{ &self.properties }

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

    /// Parses inner map element as a [`TiledMap`].
    fn parse_node(&mut self, map_node: Node) -> Result<()> {

        // Attributes
        for attribute in map_node.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            match name {
                "version" => self.version = String::from(value),
                "orientation" => self.orientation = Orientation::parse(value)?,
                "renderorder" => self.render_order = RenderOrder::from_str(value)?,
                "width" => self.width = value.parse()?,
                "height" => self.height = value.parse()?,
                "tilewidth" => self.tile_width = value.parse()?,
                "tileheight" => self.tile_height = value.parse()?,
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
                    let ctx = ParseContext { tilesets: &self.tileset_entries, infinite: self.infinite };
                    let layer = Layer::parse_tile_layer(node, &ctx)?;
                    self.layers.push(layer);
                },
                "group" => {
                    let ctx = ParseContext { tilesets: &self.tileset_entries, infinite: self.infinite };
                    let layer = Layer::parse_group_layer(node, &ctx)?;
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

/// A single tileset stored in a [`TiledMap`]`.
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

/// Information about the current state of parsing.
/// Helps reduce the number of arguments that need to be propagated.
pub(crate) struct ParseContext<'a> {
    pub tilesets: &'a [TilesetEntry],
    pub infinite: bool,
}

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


#[cfg(test)]
mod test {
    use crate::{Flip, Gid, TiledMap};

    #[test]
    fn test_finite() {

        // Loads map and gets first layer
        let xml = include_str!("test_data/finite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        let layer = map.layers()
            .iter()
            .find(|layer| layer.name() == "below")
            .unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();

        // Checks the contents of those tiles
        let tile_a_gid = tile_layer.gid_at(0, 0);
        let tile_b_gid = tile_layer.gid_at(5, 2);
        let expected = Gid::Value {
            tileset_index: 2,
            tile_id: 0,
            flip: Flip(0b0000_1000),
        };
        assert_eq!(expected, tile_a_gid);
        
        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 97,
            flip: Flip(0b0000_0000),
        };
        assert_eq!(expected, tile_b_gid);
    }

    #[test]
    fn test_infinite() {

        // Loads map and gets first layer
        let xml = include_str!("test_data/infinite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        let layer = map.layers()
            .iter()
            .find(|layer| layer.name() == "below")
            .unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();

        // Checks the contents of those tiles
        let tile_a_gid = tile_layer.gid_at(0, 0);
        let tile_b_gid = tile_layer.gid_at(5, 2);
        let expected = Gid::Value {
            tileset_index: 2,
            tile_id: 0,
            flip: Flip(0b0000_1000),
        };
        assert_eq!(expected, tile_a_gid);
        
        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 97,
            flip: Flip(0b0000_0000),
        };
        assert_eq!(expected, tile_b_gid);
    }

    #[test]
    fn test_iter() {

        // Loads map and gets first layer
        let xml = include_str!("test_data/infinite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        let layer = map.layers()
            .iter()
            .find(|layer| layer.name() == "below")
            .unwrap();
        let tile_layer = layer.as_tile_layer().unwrap();

        // Iterates over all non-null tile gids
        let mut gids = tile_layer.gids().non_null();

        // Checks first tile
        let expected_x: i32 = -4;
        let expected_y: i32 = -2;
        let expected_gid = Gid::Value {
            tileset_index: 0,
            tile_id: 0,
            flip: Flip(0b0000_0000),
        };
        assert_eq!(Some((expected_x, expected_y, expected_gid)), gids.next());

        // Checks second tile
        let expected_x: i32 = 0;
        let expected_y: i32 = 0;
        let expected_gid = Gid::Value {
            tileset_index: 2,
            tile_id: 0,
            flip: Flip(0b0000_1000),
        };
        assert_eq!(Some((expected_x, expected_y, expected_gid)), gids.next());
    }

    #[test]
    fn test_image() {

        // Loads map and gets first layer
        let xml = include_str!("test_data/infinite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        let layer = map.layers()
            .iter()
            .find(|layer| layer.name() == "background")
            .unwrap();

        // Checks the contents of those tiles
        println!("{layer:#?}");
    }

    #[test]
    fn test_flip() {

        // Loads map and gets first layer
        let xml = include_str!("test_data/flip.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        let layer = &map.layers()[0];
        let tile_layer = layer.as_tile_layer().unwrap();
        
        // Gets ids of numerous tiles
        let gid = tile_layer.gid_at(0, 0);
        let gid_rot_90 = tile_layer.gid_at(1, 0);
        let gid_rot_180 = tile_layer.gid_at(2, 0);
        let gid_rot_270 = tile_layer.gid_at(3, 0);
        let gid_other_tileset = tile_layer.gid_at(2, 1);

        // Checks that they're flipped correctly
        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 0,
            flip: Flip(0b0000_0000),
        };
        assert_eq!(expected, gid);

        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 0,
            flip: Flip(0b0000_1010),
        };
        assert_eq!(expected, gid_rot_90);

        
        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 0,
            flip: Flip(0b0000_1100),
        };
        assert_eq!(expected, gid_rot_180);

        let expected = Gid::Value {
            tileset_index: 0,
            tile_id: 0,
            flip: Flip(0b0000_0110),
        };
        assert_eq!(expected, gid_rot_270);

        let expected = Gid::Value {
            tileset_index: 1,
            tile_id: 0,
            flip: Flip(0b0000_0000),
        };
        assert_eq!(expected, gid_other_tileset);
    }
}