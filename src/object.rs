use std::str::FromStr;
use roxmltree::Node;
use crate::{parse_bool, Color, Error, Gid, Properties, Result};

/// A group of objects.
#[derive(Debug, Default)]
pub struct ObjectGroupLayer {
    color: Color,
    draw_order: DrawOrder,
    objects: Vec<Object>,
}

impl ObjectGroupLayer {

    pub fn color(&self) -> Color { self.color }
    pub fn draw_order(&self) -> DrawOrder { self.draw_order }
    pub fn objects(&self) -> &[Object] { &self.objects }

    pub(crate) fn parse(object_layer_node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in object_layer_node.attributes() {
            match attr.name() {
                "color" => result.color = attr.value().parse()?,
                "draworder" => result.draw_order = attr.value().parse()?,
                _ => {}
            }
        }
        for child in object_layer_node.children() {
            match child.tag_name().name() {
                "object" => result.objects.push(Object::parse(child)?),
                _ => {}
            }
        }
        Ok(result)
    }
}

/// A single object in an [`ObjectGroupLayer`]
#[derive(Debug, Default)]
pub struct Object {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    rotation: f32,
    gid: Option<Gid>,
    visible: bool,
    properties: Properties,
    kind: ObjectKind,
}

impl Object {
    pub fn x(&self) -> f32 { self.x }
    pub fn y(&self) -> f32 { self.y }
    pub fn width(&self) -> f32 { self.width }
    pub fn height(&self) -> f32 { self.height }
    pub fn rotation(&self) -> f32 { self.rotation }
    pub fn gid(&self) -> Option<Gid> { self.gid }
    pub fn visible(&self) -> bool { self.visible }

    fn parse(object_node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in object_node.attributes() {
            match attr.name() {
                "x" => result.x = attr.value().parse()?,
                "y" => result.y = attr.value().parse()?,
                "width" => result.width = attr.value().parse()?,
                "height" => result.height = attr.value().parse()?,
                "rotation" => result.rotation = attr.value().parse()?,
                "gid" => result.gid = Some(Gid(attr.value().parse()?)),
                "visible" => result.visible = attr.value().parse()?,
                _ => {}
            }
        }
        for child in object_node.children() {
            match child.tag_name().name() {
                "properties" => result.properties = Properties::parse(child)?,
                "ellipse" => result.kind = ObjectKind::Ellipse,
                "point" => result.kind = ObjectKind::Point,
                "polyline" => result.kind = ObjectKind::parse_polyline(child)?,
                "polygon" => result.kind = ObjectKind::parse_polygon(child)?,
                "text" => result.kind = ObjectKind::Text(Text::parse(child)?),
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


#[derive(Debug, Default)]
pub enum ObjectKind {
    #[default]
    Rectangle,
    Point,
    Ellipse,
    Polyline(Vec<(f32, f32)>),
    Polygon(Vec<(f32, f32)>),
    Text(Text),
}

impl ObjectKind {
    fn parse_polyline(node: Node) -> Result<Self> {
        let mut result = Vec::new();
        if let Some(points) = node.attribute("points") {
            parse_points(points, &mut result)?;
        }
        Ok(Self::Polyline(result))
    }

    fn parse_polygon(node: Node) -> Result<Self> {
        let mut result = Vec::new();
        if let Some(points) = node.attribute("points") {
            parse_points(points, &mut result)?;
        }
        Ok(Self::Polyline(result))
    }
}

fn parse_points(points: &str, result: &mut Vec<(f32, f32)>) -> Result<()> {
    let points = points.split(" ");
    for point in points {
        let mut xy = point.split(",");
        let x: f32 = xy.next().ok_or(Error::ParsingError)?.parse()?;
        let y: f32 = xy.next().ok_or(Error::ParsingError)?.parse()?;
        result.push((x, y));
    }
    Ok(())
}

#[derive(Debug)]
pub struct Text {
    font_family: Option<String>,
    pixel_size: f32,
    wrap: bool,
    color: Color,
    bold: bool,
    italic: bool,
    underline: bool,
    strikeout: bool,
    kerning: bool,
    halign: HAlign,
    valign: VAlign,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            font_family: None,
            pixel_size: 16.0,
            wrap: false,
            color: Color::BLACK,
            bold: false,
            italic: false,
            underline: false,
            strikeout: false,
            kerning: true,
            halign: HAlign::default(),
            valign: VAlign::default(),
        }
    }
}

impl Text {
    pub fn font_family(&self) -> &str {
        match &self.font_family {
            Some(font_family) => &font_family,
            None => "sans-serif",
        }
    }
    pub fn pixel_size(&self) -> f32 { self.pixel_size }
    pub fn wrap(&self) -> bool { self.wrap }
    pub fn color(&self) -> Color { self.color }
    pub fn bold(&self) -> bool { self.bold }
    pub fn italic(&self) -> bool { self.italic }
    pub fn underline(&self) -> bool { self.underline }
    pub fn strikeout(&self) -> bool { self.strikeout }
    pub fn kerning(&self) -> bool { self.kerning }
    pub fn halign(&self) -> HAlign { self.halign }
    pub fn valign(&self) -> VAlign { self.valign }

    pub(crate) fn parse(text_node: Node) -> Result<Self> {
        let mut result = Self::default();
        for attr in text_node.attributes() {
            match attr.name() {
                "fontfamily" => result.font_family = Some(attr.value().into()),
                "pixelsize" => result.pixel_size = attr.value().parse()?,
                "wrap" => result.wrap = parse_bool(attr.value())?,
                "color" => result.color = attr.value().parse()?,
                "bold" => result.bold = parse_bool(attr.value())?,
                "italic" => result.italic = parse_bool(attr.value())?,
                "underline" => result.underline = parse_bool(attr.value())?,
                "strikeout" => result.strikeout = parse_bool(attr.value())?,
                "kerning" => result.kerning = parse_bool(attr.value())?,
                "halign" => result.halign = attr.value().parse()?,
                "valign" => result.valign = attr.value().parse()?,
                _ => {}
            }
        }
        Ok(result)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Default, Debug)]
pub enum HAlign {
    #[default]
    Left,
    Center,
    Right,
    Justify,
}

impl FromStr for HAlign {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "left" => Ok(Self::Left),
            "center" => Ok(Self::Center),
            "right" => Ok(Self::Right),
            "justify" => Ok(Self::Justify),
            _ => Err(Error::ParsingError),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, PartialOrd, Ord, Default, Debug)]
pub enum VAlign {
    #[default]
    Top,
    Center,
    Bottom,
}

impl FromStr for VAlign {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "top" => Ok(Self::Top),
            "center" => Ok(Self::Center),
            "bottom" => Ok(Self::Bottom),
            _ => Err(Error::ParsingError),
        }
    }
}