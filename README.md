# Tiled Parser
A simple parser for [Tiled](https://www.mapeditor.org/) .tmx and .tsx files, designed for integration with game engines.

## Goals
Provide a parser that conforms to the vast majority of the .tmx and .tsx spec, starting from version 1.0 and beyond.

## Non-Goals
* Loading external tilesets referenced by maps. How external resources are loaded can depend a lot on the game engine using it, so it is intentionally left out.
* Parsing editor-specific features like wangsets, terrains, etc.
* Rendering maps.

## Supports
* Parsing .tmx and .tsx files
* Tile layers, with all encodings and compressions
* Object layers
* Image layers
* Both infinite and finite maps
* Normal tilesets
* Image collection tilesets
* Tile animations
* ...etc

## Loading a tmx file:
```rust
use tiled_parser::*;

fn main() {

    // Load the map
    let xml = include_str!("map.tmx");
    let map = Map::parse_str(xml).unwrap();

    // Iterate over layers
    for layer in map.layers() {
        match layer.kind() {

            // Iterating over tile layers
            LayerKind::TileLayer(tile_layer) => for (tile_x, tile_y, tile_gid) in tile_layer.gids() {
                let (tileset_index, tile_id) = map.tile_location_of(tile_gid).unwrap();
                let tileset_entry = &map.tileset_entries()[tileset_index];
                let tileset = match tileset_entry.kind() {
                    TilesetEntryKind::Internal(tileset) => tileset,
                    TilesetEntryKind::External(_source) => panic!("You'll need to fetch external tilesets yourself"),
                };
                let tile = tileset.tile(tile_id).unwrap();
                let props = tile.properties();
                for (name, value) in props.iter() {
                    // Do stuff with tile props
                }
            },

            // Iterating over a group layer
            LayerKind::GroupLayer(group_layer) => {
                for layer in group_layer.layers() {
                    match layer.kind() {
                        LayerKind::TileLayer(_) => {},
                        LayerKind::GroupLayer(_) => {},
                        LayerKind::ImageLayer(_) => {},
                        LayerKind::ObjectGroupLayer(_) => {},
                    }
                }
            },

            // Iterating over an image layer
            LayerKind::ImageLayer(image_layer) => {
                let image = image_layer.image();
                let source = image.source();
                let width = image.width();
                let height = image.height();
            },

            // Iterating over an object layer
            LayerKind::ObjectGroupLayer(object_layer) => {
                for object in object_layer.objects() {
                    let props = object.properties();
                    let is_steve = props.get("is_steve").unwrap();
                    let is_steve: bool = is_steve.as_bool().unwrap();
                }
            },
        }
    }
}
```
## Loading a tsx file:
```rust
use tiled_parser::*;

fn main() {
    // Load the tileset
    let xml = include_str!("tileset.tsx");
    let tileset = Tileset::parse_str(xml).unwrap();
    match tileset.image() {

        // Regular tileset
        Some(image) => {
            let image_source: &str = image.source();
            for tile in tileset.tiles() {
                let props = tile.properties();
                let TilesetRegion { x, y, width, height } = tile.region().unwrap(); // Pixel region of tile in tileset's image
            }
        },

        // Image-collection tileset
        None => {
            for tile in tileset.tiles() {
                let props = tile.properties();
                let region = tile.region(); // None, since it's an image-collection tileset.
            }
        },
    }
}
```