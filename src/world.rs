use std::io::Read;
use serde::Deserialize;
use crate::Result;

#[derive(Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct World {
    pub maps: Vec<MapRef>,
}

impl World {

    pub fn parse(mut read: impl Read) -> Result<Self> {
        let mut xml_str = String::new();
        read.read_to_string(&mut xml_str)?;
        Self::parse_str(&xml_str)
    }

    pub fn parse_str(json_str: &str) -> Result<Self> {
        let world = serde_json::de::from_str(json_str)?;
        Ok(world)
    }
}


#[derive(Deserialize, Clone, Eq, PartialEq, Debug)]
pub struct MapRef {
    #[serde(rename = "fileName")]
    pub file_name: String,
    pub height: u32,
    pub width: u32,
    pub x: i32,
    pub y: i32,
}

#[cfg(test)]
mod test {
    use crate::{World, MapRef};

    #[test]
    fn test_deserialize() {
        let json = r#"{
            "maps": [
                {
                    "fileName": "map_1.tmx",
                    "height": 384,
                    "width": 544,
                    "x": 0,
                    "y": 0
                },
                {
                    "fileName": "map_2.tmx",
                    "height": 384,
                    "width": 640,
                    "x": 544,
                    "y": 0
                }
            ],
            "onlyShowAdjacentMaps": false,
            "type": "world"
        }"#;
        let actual = World::parse_str(json).unwrap();
        let expected = World {
            maps: vec![
                MapRef {
                    file_name: "map_1.tmx".into(),
                    x: 0,
                    y: 0,
                    width: 544,
                    height: 384,
                },
                MapRef {
                    file_name: "map_2.tmx".into(),
                    x: 544,
                    y: 0,
                    width: 640,
                    height: 384,
                }
            ],
        };
        assert_eq!(expected, actual);
    }
}