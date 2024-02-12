use std::collections::HashMap;
use std::io::Read;
use roxmltree::{Document, Node};
use crate::{FillMode, Grid, ObjectAlignment, Tile, TileOffset, TileRenderSize, Result};


#[derive(Clone, Default, Debug)]
pub struct Tileset {
    pub name: String,
    pub class: String,
    pub tile_width: u32,
    pub tile_height: u32,
    pub spacing: u32,
    pub margin: u32,
    pub tile_count: u32,
    pub columns: u32,
    pub object_alignment: ObjectAlignment,
    pub tile_render_size: TileRenderSize,
    pub fill_mode: FillMode,
    pub tile_offset: TileOffset,
    pub grid: Option<Grid>,
    pub image: Option<Image>,
    pub tiles: HashMap<u32, Tile>,
}

impl Tileset {

    pub fn parse(mut read: impl Read) -> Result<Self> {
        let mut xml_str = String::new();
        read.read_to_string(&mut xml_str)?;
        Self::parse_str(&xml_str)
    }

    pub fn parse_str(xml_str: &str) -> Result<Self> {
        let mut result = Tileset::default();
        let xml_doc = Document::parse(&xml_str)?;
        let root = xml_doc.root();
        for node in root.children() {
            match node.tag_name().name() {
                "tileset" => result.parse_node(node)?,
                _ => {}
            }
        }
        Ok(result)
    }

    pub(crate) fn parse_node(&mut self, tileset_node: Node) -> Result<()> {
        for attribute in tileset_node.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            match name {
                "name" => self.name = String::from(value),
                "class" => self.class = String::from(value),
                "tilewidth" => self.tile_width = value.parse()?,
                "tileheight" => self.tile_height = value.parse()?,
                "spacing" => self.spacing = value.parse()?,
                "margin" => self.margin = value.parse()?,
                "tilecount" => self.tile_count = value.parse()?,
                "columns" => self.columns = value.parse()?,
                "objectalignment" => self.object_alignment = ObjectAlignment::parse(value)?,
                "tilerendersize" => self.tile_render_size = TileRenderSize::parse(value)?,
                "fillmode" => self.fill_mode = FillMode::parse(value)?,
                _ => {}
            }
        }
        for child in tileset_node.children() {
            let tag = child.tag_name().name();
            match tag {
                "image" => self.image = Some(Image::parse(child)?),
                "tileoffset" => self.tile_offset = TileOffset::parse(child)?,
                "grid" => self.grid = Some(Grid::parse(child)?),
                _ => {}
            }
        }
        Ok(())
    }
}

#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Image {
    pub format: String,
    pub source: String,
    pub trans: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Image {
    pub fn parse(image_node: Node) -> Result<Image> {
        let mut image = Image::default();
        for attribute in image_node.attributes() {
            let name = attribute.name();
            let value = attribute.value();
            match name {
                "format" => image.format = value.into(),
                "source" => image.source = value.into(),
                "trans" => image.trans = Some(value.into()),
                "width" => image.width = Some(value.parse()?),
                "height" => image.height = Some(value.parse()?),
                _ => {}
            }
        }
        Ok(image)
    }
}

