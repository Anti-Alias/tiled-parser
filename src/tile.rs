use crate::{Image, Properties, TilesetEntry};


#[derive(Clone, Default, Debug)]
pub struct Tile {
    properties: Properties,
    image: Option<Image>,
}

impl Tile {
    pub fn properties(&self) -> &Properties { &self.properties }
    pub fn image(&self) -> Option<&Image> { self.image.as_ref() }
}

/// Global id of a [`Tile`] within a map.
/// Includes both the tileset index and tile index for faster lookups.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Default, Debug)]
pub enum Gid {
    #[default]
    Null,
    Value {
        tileset_index: u32,
        tile_index: u32,
    },
}

impl Gid {
    /// Converts a tiled map file's gid to a [`Gid`].
    pub(crate) fn resolve(gid: u32, entries: &[TilesetEntry]) -> Self {
        for (tileset_index, tileset_entry) in entries.iter().enumerate() {
            if gid >= tileset_entry.first_gid() {
                return Gid::Value {
                    tileset_index: tileset_index as u32,
                    tile_index: gid - tileset_entry.first_gid(),
                }
            }
        }
        Self::Null
    }
}