use crate::Result;
use roxmltree::Node;

/// Image in an [`ImageLayer`](crate::ImageLayer), a [`Tileset`](crate::Tileset) or a [`Tile`](crate::Tile).
#[derive(Clone, Eq, PartialEq, Default, Debug)]
pub struct Image {
    format: String,
    source: String,
    trans: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

impl Image {
    pub fn format(&self) -> &str { &self.format }
    pub fn source(&self) -> &str { &self.source }
    pub fn trans(&self) -> Option<&str> { self.trans.as_deref() }
    pub fn width(&self) -> Option<u32> { self.width }
    pub fn height(&self) -> Option<u32> { self.height }

    pub(crate) fn parse(image_node: Node) -> Result<Image> {
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