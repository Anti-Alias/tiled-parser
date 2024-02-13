use std::io::Read;
use std::ops::Deref;
use base64::prelude::*;
use roxmltree::Node;
use flate2::read::{GzDecoder, ZlibDecoder};
use crate::{Color, Error, Gid, ParseContext, Properties, Result};


/// A layer in a [`TiledMap`](crate::map::TiledMap).
/// Can be a group, tile or object layer.
#[derive(Debug)]
pub struct Layer {
    id: u32,
    name: String,
    class: String,
    offset_x: f32,
    offset_y: f32,
    parallax_x: f32,
    parallax_y: f32,
    opacity: f32,
    visible: bool,
    locked: bool,
    tint_color: Color,
    properties: Properties,
    kind: LayerKind,
}

impl Layer {
    
    pub fn id(&self) -> u32 { self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn class(&self) -> &str { &self.class }
    pub fn offset_x(&self) -> f32 { self.offset_x }
    pub fn offset_y(&self) -> f32 { self.offset_y }
    pub fn parallax_x(&self) -> f32 { self.parallax_x }
    pub fn parallax_y(&self) -> f32 { self.parallax_y }
    pub fn opacity(&self) -> f32 { self.opacity }
    pub fn visible(&self) -> bool { self.visible }
    pub fn locked(&self) -> bool { self.locked }
    pub fn tint_color(&self) -> Color { self.tint_color }
    pub fn properties(&self) -> &Properties { &self.properties }
    pub fn kind(&self) -> &LayerKind { &self.kind }

    fn new(fields: CommonLayerFields, kind: LayerKind) -> Self {
        Self {
            id: fields.id,
            name: fields.name,
            class: fields.class,
            offset_x: fields.offset_x,
            offset_y: fields.offset_y,
            parallax_x: fields.parallax_x,
            parallax_y: fields.parallax_y,
            opacity: fields.opacity,
            visible: fields.visible,
            locked: fields.locked,
            tint_color: fields.tint_color,
            properties: fields.properties,
            kind,
        }
    }

    pub fn as_tile_layer(&self) -> Option<&TileLayer> {
        self.kind.as_tile_layer()
    }

    pub fn as_group_layer(&self) -> Option<&GroupLayer> {
        self.kind.as_group_layer()
    }

    pub(crate) fn parse_tile_layer(tile_layer_node: Node, ctx: &ParseContext) -> Result<Self> {
        let fields = CommonLayerFields::parse(tile_layer_node)?;
        let kind = LayerKind::TileLayer(TileLayer::parse(tile_layer_node, ctx)?);
        Ok(Self::new(fields, kind))
    }

    pub(crate) fn parse_group_layer(group_node: Node, ctx: &ParseContext) -> Result<Self> {
        let fields = CommonLayerFields::parse(group_node)?;
        let kind = LayerKind::GroupLayer(GroupLayer::parse(group_node, ctx)?);
        Ok(Self::new(fields, kind))
    }
}

/// The specific layer kind of a [`Layer`].
#[derive(Debug)]
pub enum LayerKind {
    TileLayer(TileLayer),
    GroupLayer(GroupLayer),
}

impl LayerKind {
    pub fn as_tile_layer(&self) -> Option<&TileLayer> {
        match self {
            LayerKind::TileLayer(tile_layer) => Some(&tile_layer),
            LayerKind::GroupLayer(_) => None,
        }
    }

    pub fn as_group_layer(&self) -> Option<&GroupLayer> {
        match self {
            LayerKind::TileLayer(_) => None,
            LayerKind::GroupLayer(group_layer) => Some(&group_layer),
        }
    }
}

/// A layer of tiles.
/// Note that mutating fields may result in panics when using helper methods.
/// Beware.
#[derive(Debug, Default)]
pub struct TileLayer {
    width: u32,
    height: u32,
    region: TileLayerRegion,
    tile_gids: Vec<Gid>,
}

impl TileLayer {
    /// Supposed width of the tile layer.
    /// Meaningless in an infinite map and should not be programmed against.
    pub fn width(&self) -> u32 { self.width }

    /// Supposed height of the tile layer.
    /// Meaningless in an infinite map and should not be programmed against.
    pub fn height(&self) -> u32 { self.width }

    /// A rectangular region that encompases all tiles in the layer.
    /// Useful when manual iteration over tiles is required.
    pub fn region(&self) -> TileLayerRegion { self.region }

    /// Gets the [`Gid`] of the tile at the specified coordinates.
    /// If out of bounds, returns [`Gid::Null`].
    pub fn gid_at(&self, x: i32, y: i32) -> Gid {
        let x = x - self.region.x;
        let y = y - self.region.y;
        let region_width = self.region.width as i32;
        let region_height = self.region.height as i32;
        if x < 0 || x >= region_width {
            return Gid::default();
        }
        if y < 0 || y >= region_height {
            return Gid::default();
        }
        self.tile_gids[(y * region_width + x) as usize]
    }

    /// Iterates over all gids in the layer, including null ones.
    /// Includes x, y coordinates (in tiles) of each tile.
    pub fn gids(&self) -> Gids<'_> {
        Gids {
            layer: self,
            x: 0,
            y: 0,
            idx: 0,
            width: self.region.width,
            total: self.tile_gids.len(),
            off_x: self.region.x,
            off_y: self.region.y,
        }
    }

    pub(crate) fn parse(layer_node: Node, ctx: &ParseContext) -> Result<Self> {
        let mut result = Self::default();
        for attr in layer_node.attributes() {
            match attr.name() {
                "width" => result.width = attr.value().parse()?,
                "height" => result.height = attr.value().parse()?,
                _ => {}
            }
        }
        let data_node = layer_node.first_element_child().ok_or(Error::InvalidLayerError)?;
        match ctx.infinite {
            true => parse_infinite_layer_data(&mut result, data_node, ctx)?,
            false => parse_finite_layer_data(&mut result, data_node, ctx)?,
        };
        Ok(result)
    }
}

/// Iterator over gids in a [`TileLayer`].
pub struct Gids<'a> {
    layer: &'a TileLayer,
    x: u32,
    y: u32,
    idx: usize,
    width: u32,
    total: usize,
    off_x: i32,
    off_y: i32,
}

impl<'a> Gids<'a> {
    /// Filters out null gids.
    pub fn non_null(self) -> NonNullGids<'a> {
        NonNullGids(self)
    }
}

impl<'a> Iterator for Gids<'a> {
    type Item = (i32, i32, Gid);
    fn next(&mut self) -> Option<Self::Item> {
        if self.idx >= self.total as usize {
            return None;
        }
        let next_x = self.x as i32 + self.off_x;
        let next_y = self.y as i32 + self.off_y;
        let next_gid = self.layer.tile_gids[self.idx];
        self.idx += 1;
        self.x += 1;
        if self.x == self.width {
            self.x = 0;
            self.y += 1;
        }
        Some((next_x, next_y, next_gid))
    }
}

/// Iterator over non-null gids in a [`TileLayer`].
pub struct NonNullGids<'a>(Gids<'a>);
impl<'a> Iterator for NonNullGids<'a> {
    type Item = (i32, i32, Gid);
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((x, y, gid)) = self.0.next() {
            if gid != Gid::Null {
                return Some((x, y, gid));
            }
        }
        None
    }
}

/// A rectangular region in a [`TileLayer`] that encompasses all tiles.
/// Useful for manual iteration.
#[derive(Copy, Clone, Eq, PartialEq, Default, Debug)]
pub struct TileLayerRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// A layer containing other [`Layer`]s.
#[derive(Default, Debug)]
pub struct GroupLayer(Vec<Layer>);
impl GroupLayer {

    pub fn layers(&self) -> &[Layer] { &self.0 }

    pub(crate) fn parse(group_node: Node, ctx: &ParseContext) -> Result<Self> {
        let mut result = Self::default();
        for node in group_node.children() {
            match node.tag_name().name() {
                "layer" => {
                    let layer = Layer::parse_tile_layer(node, &ctx)?;
                    result.0.push(layer);
                },
                "group" => {
                    let layer = Layer::parse_group_layer(node, &ctx)?;
                    result.0.push(layer)
                },
                _ => {}
            }
        }
        Ok(result)
    }
}

/// 2D storage of tile gids in an infinite tile layer.
struct Chunk {
    min_x: i32,
    min_y: i32,
    max_x: i32,
    max_y: i32,
    tile_gids: Vec<Gid>,
}

/// Fields that all layer types have in common.
struct CommonLayerFields {
    id: u32,
    name: String,
    class: String,
    offset_x: f32,
    offset_y: f32,
    parallax_x: f32,
    parallax_y: f32,
    opacity: f32,
    visible: bool,
    locked: bool,
    tint_color: Color,
    properties: Properties,
}

impl Default for CommonLayerFields {
    fn default() -> Self {
        Self {
            id: 0,
            name: "".into(),
            class: "".into(),
            offset_x: 0.0,
            offset_y: 0.0,
            parallax_x: 0.0,
            parallax_y: 0.0,
            opacity: 1.0,
            visible: true,
            locked: false,
            tint_color: Color::default(),
            properties: Properties::default(),
        }
    }
}

impl CommonLayerFields {
    fn parse(layer_node: Node) -> Result<CommonLayerFields> {
        let mut common = CommonLayerFields::default();
        for attr in layer_node.attributes() {
            match attr.name() {
                "id" => common.id = attr.value().parse()?,
                "name" => common.name = attr.value().into(),
                "class" => common.class = attr.value().into(),
                "offsetx" => common.offset_x = attr.value().parse()?,
                "offsety" => common.offset_y = attr.value().parse()?,
                "parallaxx" => common.parallax_x = attr.value().parse()?,
                "parallaxy" => common.parallax_y = attr.value().parse()?,
                "opacity" => common.opacity = attr.value().parse()?,
                "tintcolor" => common.tint_color = attr.value().parse()?,
                "visible" => common.visible = parse_bool(attr.value())?,
                "locked" => common.locked = parse_bool(attr.value())?,
                _ => {}
            }
        }
        for child in layer_node.children() {
            if child.tag_name().name() == "properties" {
                common.properties = Properties::parse(child)?;
            }
        }
        Ok(common)
    }
}

fn parse_bool(value: &str) -> Result<bool> {
    match value {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => return Err(Error::InvalidLayerError),
    }
}

/// Parses tiles in a finite layer's data node.
fn parse_finite_layer_data(layer: &mut TileLayer, data_node: Node, ctx: &ParseContext) -> Result<()> {
    let encoding = data_node.attribute("encoding");
    let compression = data_node.attribute("compression");
    let tile_gids = data_node.text().ok_or(Error::InvalidLayerError)?.trim();
    let tile_gids = parse_tile_gids(tile_gids, encoding, compression)?;
    let tile_gids = tile_gids.into_iter().map(|gid_int| Gid::resolve(gid_int, ctx.tilesets)).collect();
    layer.tile_gids = tile_gids;
    layer.region.width = layer.width;
    layer.region.height = layer.height;
    Ok(())
}

/// Parses tiles in an infinite layer's data node.
fn parse_infinite_layer_data(layer: &mut TileLayer, data_node: Node, ctx: &ParseContext) -> Result<()> {
    let encoding = data_node.attribute("encoding");
    let compression = data_node.attribute("compression");

    // Collects chunks
    let mut chunks = Vec::new();
    let mut global_min_x = i32::MAX;
    let mut global_min_y = i32::MAX;
    let mut global_max_x = i32::MIN;
    let mut global_max_y = i32::MIN;
    for chunk_node in data_node.children() {
        if !chunk_node.has_tag_name("chunk") { continue };
        let mut x: i32 = 0;
        let mut y: i32 = 0;
        let mut width: u32 = 0;
        let mut height: u32 = 0;
        for attr in chunk_node.attributes() {
            match attr.name() {
                "x" => x = attr.value().parse()?,
                "y" => y = attr.value().parse()?,
                "width" => width = attr.value().parse()?,
                "height" => height = attr.value().parse()?,
                _ => {}
            }
            let x2 = x + width as i32;
            let y2 = y + height as i32;
            global_min_x = global_min_x.min(x);
            global_min_y = global_min_y.min(y);
            global_max_x = global_max_x.max(x2);
            global_max_y = global_max_y.max(y2);
        }
        let max_x = x + width as i32;
        let max_y = y + height as i32;
        let tile_gids = chunk_node
            .text()
            .ok_or(Error::InvalidLayerError)?.trim();
        let tile_gids = parse_tile_gids(tile_gids, encoding, compression)?;
        let tile_gids: Vec<Gid> = tile_gids.into_iter().map(|gid_int| Gid::resolve(gid_int, ctx.tilesets)).collect();
        chunks.push(Chunk { min_x: x, min_y: y, max_x, max_y, tile_gids });
    }

    // Allocates vec to fit tile gids in all chunks.
    let raw_width = (global_max_x - global_min_x) as u32;
    let raw_height = (global_max_y - global_min_y) as u32;
    let mut raw_tile_gids = vec![Gid::Null; (raw_width * raw_height) as usize];

    // Composites chunks to vec.
    for chunk in chunks {
        let chunk_width = chunk.max_x - chunk.min_x;
        for global_y in chunk.min_y..chunk.max_y {
            for global_x in chunk.min_x..chunk.max_x {
                let raw_idx = {
                    let raw_x = global_x - global_min_x;
                    let raw_y = global_y - global_min_y;
                    (raw_y * raw_width as i32 + raw_x) as usize
                };
                let chunk_idx = {
                    let chunk_x = global_x - chunk.min_x;
                    let chunk_y = global_y - chunk.min_y;
                    (chunk_y * chunk_width as i32 + chunk_x) as usize
                };
                raw_tile_gids[raw_idx] = chunk.tile_gids[chunk_idx];
            }
        }
    }

    // Writes to layer
    layer.tile_gids = raw_tile_gids;
    layer.region.x = global_min_x;
    layer.region.y = global_min_y;
    layer.region.width = raw_width;
    layer.region.height = raw_height;
    Ok(())
}

fn parse_tile_gids(layer_data: &str, encoding: Option<&str>, compression: Option<&str>) -> Result<Vec<u32>> {
    match (encoding, compression) {
        (Some("csv"), None) => parse_csv(layer_data),
        (Some("base64"), None) => {
            let decoded = decode_base64(layer_data.as_bytes())?;
            let parsed = parse_bytes(decoded.deref())?;
            Ok(parsed)
        },
        (Some("base64"), Some("gzip")) => {
            let decoded = decode_base64(layer_data.as_bytes()).map_err(|_| Error::DecodeLayerError)?;
            let decompressed = GzDecoder::new(decoded.deref());
            let parsed = parse_bytes(decompressed)?;
            Ok(parsed)
        },
        (Some("base64"), Some("zlib")) => {
            let decoded = decode_base64(layer_data.as_bytes()).map_err(|_| Error::DecodeLayerError)?;
            let decompressed = ZlibDecoder::new(decoded.deref());
            let parsed = parse_bytes(decompressed)?;
            Ok(parsed)
        },
        (Some("base64"), Some("zstd")) => {
            let decoded = decode_base64(layer_data.as_bytes())?;
            let decompressed = zstd::stream::Decoder::new(decoded.deref()).map_err(|_| Error::DecodeLayerError)?;
            let parsed = parse_bytes(decompressed)?;
            Ok(parsed)
        },
        _ => return Err(Error::DecodeLayerError),
    }
}

fn parse_csv(csv: &str) -> Result<Vec<u32>> {
    let mut result: Vec<u32> = Vec::new();
    for s in csv.split(',') {
        let s = s.trim();
        result.push(s.parse()?)
    }
    Ok(result)
}

fn decode_base64(encoded_bytes: &[u8]) -> Result<Vec<u8>> {
    BASE64_STANDARD.decode(&encoded_bytes).map_err(|_| Error::DecodeLayerError)
}

fn parse_bytes(mut read: impl Read) -> Result<Vec<u32>> {
    let mut result: Vec<u32> = Vec::new();
    let mut bytes: [u8; 4] = [0; 4];
    while read.read(&mut bytes)? != 0 {
        let tile_gid = u32::from_le_bytes(bytes);
        result.push(tile_gid);
    }
    Ok(result)
}
