use serde::{Serialize, Deserialize};
use strum::EnumString;

use crate::ObjectReference;

#[derive(Debug, Default, EnumString, Serialize, Deserialize)]
pub enum PropertyValue {
  Bool(bool),
  Int8(i8),
  Int(i32),
  UInt32(u32),
  Int64(i64),
  UInt64(u64),
  Float(f32),
  Double(f64),
  #[strum(serialize = "Str", serialize = "Name")]
  String(String),
  #[strum(serialize = "Object", serialize = "Interface")]
  ObjectReference(ObjectReference),
  Enum,
  Byte,
  Text,
  Array,
  Map,
  Set,
  Struct,
  #[default]
  None
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Property {
  pub name: String,
  pub r#type: String,
  pub size: i32,
  pub index: i32,
  pub guid: Option<String>,
  pub value: PropertyValue,
}
