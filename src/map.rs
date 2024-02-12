use std::io::Read;
use roxmltree::{Document, Node};
use crate::{Error, Layer, Orientation, Properties, Result, Tileset};

#[derive(Default, Debug)]
pub struct TiledMap {
    pub version: String,
    pub orientation: Orientation,
    pub render_order: RenderOrder,
    pub width: u32, 
    pub height: u32,
    pub tile_width: u32,
    pub tile_height: u32,
    pub tilesets: Vec<TilesetEntry>,
    pub infinite: bool,
    pub layers: Vec<Layer>,
    pub properties: Properties,
}

impl TiledMap {

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
                "tileset" => self.tilesets.push(TilesetEntry::parse(node)?),
                "properties" => self.properties = Properties::parse(node)?,
                // Note: According to spec, <tileset> elements always appear before <layer>, and <group> elements,
                // So the tilesets passed in are already complete.
                "layer" => {
                    let ctx = ParseContext { tilesets: &self.tilesets, infinite: self.infinite };
                    let layer = Layer::parse_tile_layer(node, &ctx)?;
                    self.layers.push(layer);
                },
                "group" => {
                    let ctx = ParseContext { tilesets: &self.tilesets, infinite: self.infinite };
                    let layer = Layer::parse_group_layer(node, &ctx)?;
                    self.layers.push(layer)
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
    pub first_gid: u32,
    pub kind: TilesetEntryKind,
}

impl TilesetEntry {

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
    use crate::TiledMap;

    #[test]
    fn test_finite() {
        let xml = include_str!("test_data/finite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        println!("{map:#?}");
    }

    #[test]
    fn test_infinite() {
        let xml = include_str!("test_data/infinite.tmx");
        let map = TiledMap::parse_str(xml).unwrap();
        println!("{map:#?}");
    }
}