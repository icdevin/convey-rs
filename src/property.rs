use std::collections::HashMap;

use serde::{Serialize, Deserialize};
use strum::EnumString;

use crate::{ObjectReference, Quaternion, Vector, Vector2D, Vector4};

/// A container is needed for these types because many fields are "upgraded"
/// from floats to doubles with new file versions, and other times types
/// may contain multiple inner types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Numeric {
  Int(i32),
  Long(i64),
  Float(f32),
  Double(f64),
}

#[derive(Clone, Debug, EnumString, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
  Bool(u8),
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
  Object(ObjectReference),
  Enum(HashMap<String, String>),
  Byte(ByteProperty),
  Text(TextProperty),
  Array(ArrayProperty),
  Map(MapProperty),
  Set(SetProperty),
  Struct((String, StructPropertyValue)),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Property {
  pub name: String,
  pub r#type: String,
  pub size: i32,
  pub index: i32,
  pub guid: Option<String>,
  pub value: PropertyValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapPropertyKey {
  Int(i32),
  Long(i64),
  String(String),
  Object(ObjectReference),
  IntVector(Vector<i32>),
  FloatVector(Vector<f32>),
  DoubleVector(Vector<f64>),
  Properties(Vec<Property>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MapPropertyValue {
  Byte(u8),
  Bool(u8),
  String(String),
  Int(i32),
  Long(i64),
  Float(f32),
  Double(f64),
  Text(TextProperty),
  Object(ObjectReference),
  Struct(Vec<Property>),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct MapProperty {
  pub key_type: String,
  pub value_type: String,
  pub mode_type: i32,
  pub unk_mode_1: Option<String>,
  pub unk_mode_2: Option<String>,
  pub unk_mode_3: Option<String>,
  pub m_normal_index: Option<i32>,
  pub m_overflow_index: Option<i32>,
  pub m_filter_index: Option<i32>,
  pub unk_float_1: Option<Numeric>,
  pub unk_float_2: Option<Numeric>,
  pub unk_float_3: Option<Numeric>,
  pub unk_float_4: Option<Numeric>,
  pub unk_str_1: Option<String>,
  pub keys: Vec<MapPropertyKey>,
  pub values: Vec<MapPropertyValue>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SetPropertyValue {
  Int(i32),
  UInt32(u32),
  Object(ObjectReference),
  String(String),
  Vector(Vector<f32>),
  FINNetworkTrace(FINNetworkTrace),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SetProperty {
  pub r#type: String,
  pub values: Vec<SetPropertyValue>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StructPropertyBox<T> {
  pub min: Vector<T>,
  pub max: Vector<T>,
  pub is_valid: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StructPropertyRailroadTrackPosition {
  pub object: ObjectReference,
  pub offset: f32,
  pub forward: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StructPropertyInventoryItem {
  pub unk_int_1: i32,
  pub item_name: String,
  pub object: ObjectReference,
  pub property: Box<Property>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InventoryStack {
  pub unk_str_1: String,
  pub unk_str_2: String,
  pub unk_int_1: i32,
  pub unk_int_2: i32,
  pub unk_struct_1: (String, StructPropertyValue),
  pub unk_str_3: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ItemAmount {
  pub unk_int_1: i32,
  pub unk_str_1: String,
  pub unk_int_2: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FINLuaProcessorStateStorageStructValue {
  Vector(Vector<f32>),
  LinearColor(Color<f32>),
  InventoryStack(InventoryStack),
  ItemAmount(ItemAmount),
  FINTrackGraph(FINNetworkTrace, i32),
  FINGPUT1Buffer(FINGPUT1Buffer),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FINLuaProcessorStateStorageStruct {
  pub unk_int_1: i32,
  pub class_name: String,
  pub value: FINLuaProcessorStateStorageStructValue,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FINLuaProcessorStateStorage {
  pub trace: Vec<FINNetworkTrace>,
  pub reference: Vec<ObjectReference>,
  pub thread: String,
  pub globals: String,
  pub structs: Vec<FINLuaProcessorStateStorageStruct>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FrameRange {
  pub begin: i64,
  pub end: i64,
}


#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum StructPropertyValue {
  Color(Color<u8>),
  LinearColor(Color<f32>),
  FloatVector(Vector<f32>),
  DoubleVector(Vector<f64>),
  IntVector2D(Vector2D<i32>),
  FloatVector2D(Vector2D<f32>),
  DoubleVector2D(Vector2D<f64>),
  IntVector4(Vector4<i32>),
  DoubleVector4(Vector4<f64>),
  FloatQuaternion(Quaternion<f32>),
  DoubleQuaternion(Quaternion<f64>),
  Box(StructPropertyBox<f64>),
  RailroadTrackPosition(StructPropertyRailroadTrackPosition),
  TimerHandle(String),
  GUID(String),
  InventoryItem(StructPropertyInventoryItem),
  FluidBox(f32),
  SlateBrush(String),
  DateTime(i64),
  FINNetworkTrace(FINNetworkTrace),
  FINLuaProcessorStateStorage(FINLuaProcessorStateStorage),
  FICFrameRange(FrameRange),
  IntPoint(Vector2D<i32>),
  Properties(Vec<Property>),
  #[default]
  None
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct InventoryItem {
  pub unk_int_1: i32,
  pub item_name: String,
  pub level_name: String,
  pub path_name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FINNetworkTrace {
  pub level_name: String,
  pub path_name: String,
  pub prev: Option<Box<FINNetworkTrace>>,
  pub step: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FINGPUT1Buffer {
  pub x: i32,
  pub y: i32,
  pub size: i32,
  pub name: String,
  pub r#type: String,
  pub length: i32,
  pub buffer: Vec<FINGPUT1BufferPixel>,
  pub unk_str_1: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FINGPUT1BufferPixel {
  pub character: String,
  pub foreground_color: Color<f32>,
  pub background_color: Color<f32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Color<T> {
  pub red: T,
  pub green: T,
  pub blue: T,
  pub alpha: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArrayPropertyStructValue {
  InventoryItem(InventoryItem),
  GUID(String),
  FINNetworkTrace(FINNetworkTrace),
  Vector(Vector<f64>),
  LinearColor(Color<f32>),
  FINGPUT1BufferPixel(FINGPUT1BufferPixel),
  Properties(Vec<Property>),
  #[default]
  None,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArrayPropertyStruct {
  pub size_bytes: i32,
  pub r#type: String,
  pub guid1: i32,
  pub guid2: i32,
  pub guid3: i32,
  pub guid4: i32,
  pub value: ArrayPropertyStructValue,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArrayPropertyValue {
  Byte(u8),
  Bool(u8),
  Int(i32),
  Long(i64),
  Float(f32),
  Enum(String),
  Str(String),
  Text(TextProperty),
  Object(ObjectReference),
  Struct(ArrayPropertyStructValue),
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArrayProperty {
  pub r#type: String,
  pub struct_meta: Option<ArrayPropertyStruct>,
  pub elements: Vec<ArrayPropertyValue>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ByteProperty {
  pub r#type: String,
  pub byte_value: Option<u8>,
  pub string_value: Option<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BaseHistory {
  pub namespace: String,
  pub key: String,
  pub value: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Argument {
  pub name: String,
  pub value_type: u8,
  pub value: Box<TextProperty>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ArgumentHistory {
  pub source_format: Box<TextProperty>,
  pub num_arguments: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct StringTableEntryHistory {
  pub table_id: String,
  pub text_key: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TransformHistory {
  pub source_text: Box<TextProperty>,
  pub transform_type: u8,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct NoneHistory {
  pub has_culture_invariant_string: i32,
  pub value: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TextPropertyHistory {
  BaseHistory(BaseHistory),
  ArgumentHistory(ArgumentHistory),
  StringTableEntryHistory(StringTableEntryHistory),
  TransformHistory(TransformHistory),
  NoneHistory(NoneHistory),
  #[default]
  None,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TextProperty {
  pub flags: i32,
  pub history_type: u8,
  pub value: TextPropertyHistory,
}
