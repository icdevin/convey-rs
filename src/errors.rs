use std::{io, string::{FromUtf16Error, FromUtf8Error}};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ParseError {
  #[error("Read error: {0}")]
  Read(#[from] io::Error),

  #[error("Unsupported file version: {0} (minimum: {1}")]
  UnsupportedFileVersion(i32, i32),

  #[error("UTF-8 encoding error: {0}")]
  UTF8(#[from] FromUtf8Error),

  #[error("UTF-16 encoding error: {0}")]
  UTF16(#[from] FromUtf16Error),

  #[error("Missing object header in level: {0}")]
  MissingObjectHeader(String),

  #[error("Unknown object type: {0}")]
  UnknownObject(i32),

  #[error("Unknown player type: {0}")]
  UnknownPlayerType(u8),

  #[error("Unknown player ID type: {0}")]
  UnknownPlayerIDType(u8),

  #[error("Object longer than specified: {0}")]
  ObjectLength(String),

  #[error("Unknown property type: {0}")]
  UnknownPropertyType(String),

  #[error("Unknown array element type: {0}")]
  UnknownArrayElementType(String),

  #[error("Unknown map key type: {0}")]
  UnknownMapKeyType(String),

  #[error("Unknown map value type: {0}")]
  UnknownMapValueType(String),

  #[error("Unknown set type: {0}")]
  UnknownSetType(String),

  #[error("Unknown text argument value type: {0}")]
  UnknownTextArgumentValueType(u8),

  #[error("Unknown text history type: {0}")]
  UnknownTextHistoryType(u8),

  #[error("Missing inventory item property: {0}")]
  MissingInventoryItemProperty(String),

  #[error("Unknown Lua processor state storage struct type: {0}")]
  UnknownLuaProcessorStateStorageStructType(String),
}
