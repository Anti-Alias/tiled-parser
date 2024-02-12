use std::io::Read;
use roxmltree::{Document, Node};
use crate::{FillMode, Grid, ObjectAlignment, Result, TileOffset, TileRenderSize};


#[derive(Clone, Default, Debug)]
pub struct Tileset {
    name: String,
    class: String,
    tile_width: u32,
    tile_height: u32,
    spacing: u32,
    margin: u32,
    tile_count: u32,
    columns: u32,
    object_alignment: ObjectAlignment,
    tile_render_size: TileRenderSize,
    fill_mode: FillMode,
    tile_offset: TileOffset,
    grid: Option<Grid>,
    image: Option<Image>,
}

impl Tileset {

    pub fn name(&self) -> &str { &self.name }
    pub fn class(&self) -> &str { &self.class }
    pub fn tile_width(&self) -> u32 { self.tile_width }
    pub fn tile_height(&self) -> u32 { self.tile_height }
    pub fn spacing(&self) -> u32 { self.spacing }
    pub fn margin(&self) -> u32 { self.margin }
    pub fn tile_count(&self) -> u32 { self.tile_count }
    pub fn columns(&self) -> u32 { self.columns }
    pub fn object_alignment(&self) -> ObjectAlignment { self.object_alignment }
    pub fn tile_render_size(&self) -> TileRenderSize { self.tile_render_size }
    pub fn fill_mode(&self) -> FillMode { self.fill_mode }
    pub fn tile_offset(&self) -> TileOffset { self.tile_offset }
    pub fn grid(&self) -> Option<Grid> { self.grid }
    pub fn image(&self) -> Option<&Image> { self.image.as_ref() }

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

