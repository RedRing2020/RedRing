use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttributeValueKind {
    String,
    Integer,
    Float,
    Boolean,
    Color,
    Binary,
}

impl AttributeValueKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::Integer => "integer",
            Self::Float => "float",
            Self::Boolean => "boolean",
            Self::Color => "color",
            Self::Binary => "binary",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AttributeValue {
    String(String),
    Integer(i64),
    Float(f64),
    Boolean(bool),
    Color([f32; 4]),
    Binary(Vec<u8>),
}

impl AttributeValue {
    pub const fn kind(&self) -> AttributeValueKind {
        match self {
            Self::String(_) => AttributeValueKind::String,
            Self::Integer(_) => AttributeValueKind::Integer,
            Self::Float(_) => AttributeValueKind::Float,
            Self::Boolean(_) => AttributeValueKind::Boolean,
            Self::Color(_) => AttributeValueKind::Color,
            Self::Binary(_) => AttributeValueKind::Binary,
        }
    }
}
