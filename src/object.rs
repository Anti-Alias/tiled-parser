use std::str::FromStr;
use roxmltree::Node;
use crate::{Error, Result};


/// A group of objects.
#[derive(Debug, Default)]
pub struct ObjectGroupLayer {
    draw_order: DrawOrder,
}

impl ObjectGroupLayer {

    pub fn draw_order(&self) -> DrawOrder { self.draw_order }

    pub(crate) fn parse(object_layer_node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in object_layer_node.attributes() {
            match attr.name() {
                "draworder" => result.draw_order = attr.value().parse()?,
                _ => {}
            }
        }
        Ok(result)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Default, PartialOrd, Ord, Hash, Debug)]
pub enum DrawOrder {
    #[default]
    Index,
    TopDown,
}

impl FromStr for DrawOrder {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "index" => Ok(Self::Index),
            "topdown" => Ok(Self::TopDown),
            _ => Err(Error::ParsingError),
        }
    }
}
