use std::collections::HashMap;
use roxmltree::Node;
use crate::{Color, Result, Error};

/// A set of properties.
#[derive(Clone, Default, Debug)]
pub struct Properties(HashMap<String, PropertyValue>);
impl Properties {

    pub fn get(&self, name: &str) -> Option<&PropertyValue> {
        self.0.get(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    pub(crate) fn parse(properties_node: Node) -> Result<Self> {
        let mut result = Self::default();
        for child_node in properties_node.children() {
            let name = child_node.tag_name().name();
            match name {
                "property" => result.parse_property(child_node)?,
                _ => {},
            }
        }
        Ok(result)
    }

    fn parse_property(&mut self, property_node: Node) -> Result<()> {
        let name = match property_node.attribute("name") {
            Some(name) => name,
            None => return Err(Error::ParsingError),
        };
        let str_value = match property_node.attribute("value") {
            Some(value) => value,
            None => return Err(Error::ParsingError),
        };
        let str_type = property_node.attribute("type");
        let value = PropertyValue::parse(str_value, str_type)?;
        self.0.insert(name.into(), value);
        Ok(())
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum PropertyValue {
    String(String),
    Int(i32),
    Float(f32),
    Bool(bool),
    Color(Color),
    File(String),
}

impl PropertyValue {

    fn parse(value: &str, type_name: Option<&str>) -> Result<Self> {
        match type_name {
            Some("string") | None => Ok(Self::String(value.into())),
            Some("int") => Ok(Self::Int(value.parse()?)),
            Some("float") => Ok(Self::Float(value.parse()?)),
            Some("bool") => Ok(Self::Bool(value.parse()?)),
            Some("color") => Ok(Self::Color(value.parse()?)),
            Some("file") => Ok(Self::File(value.into())),
            Some(_) => Err(Error::ParsingError)
        }
    }

    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(str) => Some(&str),
            _ => None,
        }
    }
    pub fn as_int(&self) -> Option<i32> {
        match self {
            PropertyValue::Int(int) => Some(*int),
            _ => None,
        }
    }
    pub fn as_float(&self) -> Option<f32> {
        match self {
            PropertyValue::Float(float) => Some(*float),
            _ => None,
        }
    }
    pub fn as_number(&self) -> Option<f32> {
        match self {
            PropertyValue::Float(float) => Some(*float),
            PropertyValue::Int(int) => Some(*int as f32),
            _ => None,
        }
    }
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(bool) => Some(*bool),
            _ => None,
        }
    }
    pub fn as_color(&self) -> Option<Color> {
        match self {
            PropertyValue::Color(color) => Some(*color),
            _ => None,
        }
    }
    pub fn as_file(&self) -> Option<&str> {
        match self {
            PropertyValue::File(file) => Some(&file),
            _ => None,
        }
    }
}