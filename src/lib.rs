use std::collections::HashMap;

use std::fs;
use std::io::{self, Read, Seek};
use std::path::Path;
use std::result;
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use env_logger::{self, Env};
use flate2::bufread::ZlibDecoder;
use log::{debug, warn};

pub mod errors;
pub mod property;
pub mod save;

use crate::errors::ParseError;
use crate::property::*;
use crate::save::*;

pub type Result<T> = result::Result<T, ParseError>;

/// Establishes a minimum version to support which is currently the latest
/// version as of the time of this library's creation
pub const MIN_SAVE_FILE_VERSION: i32 = 42;

/// The only entry point to this library. Given a Path or something that can be
/// converted into one, attempts to read the file at that path and then reads
/// its various components byte-by-byte to build up its own representation of
/// the data within
pub fn read_file<P: AsRef<Path>>(path: P) -> result::Result<Save, ParseError> {
  // Sets up the logger
  let env = Env::default();
  let _ = env_logger::try_init_from_env(env);

  // Reads the file as a byte array and establishes a cursor
  // in order to read byte-by-byte
  let file_bytes = fs::read(&path)?;
  let file_size_bytes = file_bytes.len();
  let mut cursor = io::Cursor::new(file_bytes);

  // Begins by reading the file header bytes and checking the version of the
  // file against the minimum supported version
  let header = cursor.read_header::<LittleEndian>()?;
  if header.save_file_version < MIN_SAVE_FILE_VERSION {
    return Err(ParseError::UnsupportedFileVersion(header.save_file_version, MIN_SAVE_FILE_VERSION))
  }

  // Reads the chunk headers and the zlib-compressed chunk body bytes
  // and drop the cursor as its no longer needed
  let chunks = cursor.read_chunks::<LittleEndian>(file_size_bytes as u64)?;
  drop(cursor);

  // Decompresses each chunk's body bytes and appends it to a new byte array
  // containing all chunk's decompressed body bytes in order
  let mut body_bytes: Vec<u8> = vec![];
  for chunk_bytes in chunks {
    let mut z = ZlibDecoder::new(&chunk_bytes[..]);
    z.read_to_end(&mut body_bytes)?;
  }

  // Establishes a new cursor for the decompressed body bytes
  let mut body_cursor = io::Cursor::new(body_bytes);

  // TODO: What is this?
  body_cursor.seek_relative(8)?;

  // Reads the partitions and levels and returns the complete save file object
  // representation
  let partitions = body_cursor.read_partitions::<LittleEndian>()?;
  let levels = body_cursor.read_levels::<LittleEndian>(&header)?;

  Ok(Save {
    header,
    partitions,
    levels,
  })
}

/// Extends `byteorder`'s `ReadBytesExt` (which itself extends `io::Read`)
/// and `io::Seek` to build a robust byte reader with many great
/// utility functions needed to support the custom save file format
pub trait ReadSaveFileBytes: ReadBytesExt + Seek {
  /// Reads a quaternion with values as 32-bit floats
  fn read_quaternion<E: ByteOrder>(&mut self) -> Result<Quaternion<f32>> {
    Ok(Quaternion {
      x: self.read_f32::<E>()?,
      y: self.read_f32::<E>()?,
      z: self.read_f32::<E>()?,
      w: self.read_f32::<E>()?,
    })
  }

  /// Reads a quaternion with values as 64-bit floats
  fn read_quaternion_double<E: ByteOrder>(&mut self) -> Result<Quaternion<f64>> {
    Ok(Quaternion {
      x: self.read_f64::<E>()?,
      y: self.read_f64::<E>()?,
      z: self.read_f64::<E>()?,
      w: self.read_f64::<E>()?,
    })
  }

  /// Reads a 2D vector with values as 64-bit floats
  fn read_vector2d_double<E: ByteOrder>(&mut self) -> Result<Vector2D<f64>> {
    Ok(Vector2D {
      x: self.read_f64::<E>()?,
      y: self.read_f64::<E>()?,
    })
  }

  /// Reads a 2D vector with values as 32-bit integers
  fn read_vector2d_int<E: ByteOrder>(&mut self) -> Result<Vector2D<i32>> {
    Ok(Vector2D {
      x: self.read_i32::<E>()?,
      y: self.read_i32::<E>()?,
    })
  }

  /// Reads a 3D vector with values as 32-bit floats
  fn read_vector<E: ByteOrder>(&mut self) -> Result<Vector<f32>> {
    Ok(Vector {
      x: self.read_f32::<E>()?,
      y: self.read_f32::<E>()?,
      z: self.read_f32::<E>()?,
    })
  }

  /// Reads a 3D vector with values as 64-bit floats
  fn read_vector_double<E: ByteOrder>(&mut self) -> Result<Vector<f64>> {
    Ok(Vector {
      x: self.read_f64::<E>()?,
      y: self.read_f64::<E>()?,
      z: self.read_f64::<E>()?,
    })
  }

  /// Reads a 3D vector with values as 32-bit integers
  fn read_vector_int<E: ByteOrder>(&mut self) -> Result<Vector<i32>> {
    Ok(Vector {
      x: self.read_i32::<E>()?,
      y: self.read_i32::<E>()?,
      z: self.read_i32::<E>()?,
    })
  }

  /// Reads a 4D vector with values as 64-bit floats
  fn read_vector4_double<E: ByteOrder>(&mut self) -> Result<Vector4<f64>> {
    Ok(Vector4 {
      a: self.read_f64::<E>()?,
      b: self.read_f64::<E>()?,
      c: self.read_f64::<E>()?,
      d: self.read_f64::<E>()?,
    })
  }

  /// Reads a 4D vector with values as 32-bit integers
  fn read_vector4_int<E: ByteOrder>(&mut self) -> Result<Vector4<i32>> {
    Ok(Vector4 {
      a: self.read_i32::<E>()?,
      b: self.read_i32::<E>()?,
      c: self.read_i32::<E>()?,
      d: self.read_i32::<E>()?,
    })
  }

  /// Reads an RGB color with alpha channel with values as 32-bit floats
  fn read_color<E: ByteOrder>(&mut self) -> Result<Color<f32>> {
    Ok(Color {
      red: self.read_f32::<E>()?,
      green: self.read_f32::<E>()?,
      blue: self.read_f32::<E>()?,
      alpha: self.read_f32::<E>()?,
    })
  }

  /// Reads an RGB color with alpha channel with values as bytes
  fn read_color_byte(&mut self) -> Result<Color<u8>> {
    Ok(Color {
      red: self.read_u8()?,
      green: self.read_u8()?,
      blue: self.read_u8()?,
      alpha: self.read_u8()?,
    })
  }

  /// Reads a specified number of bytes and attempts to parse them as a
  /// UTF-16 string
  fn read_hex<E: ByteOrder>(&mut self, len: usize) -> Result<String> {
    let mut vec: Vec<u16> = vec![0; len];
    self.read_u16_into::<E>(&mut vec)?;
    Ok(String::from_utf16(&vec)?)
  }

  /// Reads a string whose length and encoding are specified by a prefixed
  /// byte:
  ///
  /// - If the length byte is > 0, the following string is UTF-8 encoded
  /// - If the length byte is < 0, the following string is UTF-16 encoded
  /// - If it == 0, the string is empty
  ///
  /// The string is terminated by a null termination byte (/0) which is removed
  fn read_length_prefixed_string<E: ByteOrder>(&mut self) -> Result<String> {
    let len = self.read_i32::<E>()?;

    let mut string: String;
    if len > 0 {
      let mut dst: Vec<u8> = vec![0; len as usize];
      self.read_exact(&mut dst)?;
      string = String::from_utf8(dst)?;
      string.pop(); // Removes the null termination byte
    } else if len < 0 {
      string = self.read_hex::<E>(len.abs() as usize)?;
      string.pop(); // Removes the null termination byte
    } else {
      string = String::from("");
    }

    Ok(string)
  }

  /// Similar to the above except the string is skipped over instead of
  /// parsed; used for unknown or redundant fields
  fn seek_length_prefixed_string<E: ByteOrder>(&mut self) -> Result<()> {
    let len = self.read_i32::<E>()?;
    self.seek_relative(len.abs() as i64)?;
    Ok(())
  }

  /// Reads the file header; some of its fields, particularly the file version
  /// and the map name, are used to drive conditional parsing in other areas
  fn read_header<E: ByteOrder>(&mut self) -> Result<Header> {
    let mut header = Header::default();

    header.save_header_version = self.read_i32::<E>()?;
    header.save_file_version = self.read_i32::<E>()?;
    header.build_version = self.read_i32::<E>()?;
    header.map_name = self.read_length_prefixed_string::<E>()?;
    header.map_options = self.read_length_prefixed_string::<E>()?;
    header.session_name = self.read_length_prefixed_string::<E>()?;
    header.played_seconds = self.read_i32::<E>()?;
    header.save_timestamp = self.read_i64::<E>()?;
    header.session_visibility = self.read_i8()?;
    header.editor_object_version = self.read_i32::<E>()?;
    header.mod_metadata = self.read_length_prefixed_string::<E>()?;
    header.mod_flags = self.read_i32::<E>()?;
    header.save_identifier = self.read_length_prefixed_string::<E>()?;
    header.is_partitioned_world = self.read_i32::<E>()?;
    header.saved_data_hash = self.read_hex::<E>(10)?;
    header.is_creative_mode_enabled = self.read_i32::<E>()?;

    Ok(header)
  }

  /// Reads a chunk header and its compressed body, performing some assertions
  /// on some fixed, well-known values
  fn read_chunk<E: ByteOrder>(&mut self) -> Result<Vec<u8>> {
    // Represents the Unreal Engine package signature (The hex value "9E2A83C1" as a u32)
    self.seek_relative(4)?;

    // Padding
    self.seek_relative(4)?;

    // Represents the max chunk size and has historically been a static
    // value of 131,072 (as a u32)
    self.seek_relative(4)?;

    // Padding
    self.seek_relative(5)?;

    let current_chunk_size = self.read_u32::<E>()?;

    // Padding
    self.seek_relative(28)?;

    let mut chunk_bytes: Vec<u8> = vec![0; current_chunk_size as usize];
    self.read_exact(&mut chunk_bytes)?;

    Ok(chunk_bytes)
  }

  /// Reads a chunk at a time until reaching the specified stop byte (which is
  /// the end of the file)
  fn read_chunks<E: ByteOrder>(&mut self, stop_byte: u64) -> Result<Vec<Vec<u8>>> {
    let mut chunks: Vec<Vec<u8>> = vec![];

    while self.stream_position()? < stop_byte {
      let chunk_bytes = self.read_chunk::<E>()?;
      chunks.push(chunk_bytes);
    }

    Ok(chunks)
  }

  /// Reads the partition objects which start the main body and come before the
  /// level data
  fn read_partitions<E: ByteOrder>(&mut self) -> Result<Partitions> {
    let mut partitions = Partitions::default();
    let num_partitions = self.read_i32::<E>()?;
    partitions.unk_str_1 = self.read_length_prefixed_string::<E>()?;
    partitions.unk_num_1 = self.read_i64::<E>()?;
    partitions.unk_num_2 = self.read_i32::<E>()?;
    partitions.unk_str_2 = self.read_length_prefixed_string::<E>()?;
    partitions.unk_num_3 = self.read_i32::<E>()?;

    for _ in 1..num_partitions {
      let key = self.read_length_prefixed_string::<E>()?;

      let mut partition = Partition::default();
      partition.unk_num_1 = self.read_i32::<E>()?;
      partition.unk_num_2 = self.read_i32::<E>()?;

      let num_levels = self.read_i32::<E>()?;
      for _ in 0..num_levels {
        let level_key = self.read_length_prefixed_string::<E>()?;
        let level_value = self.read_u32::<E>()?;
        partition.levels.insert(level_key, level_value);
      }

      partitions.partitions.insert(key, Partition::default());
    }
    Ok(partitions)
  }

  /// Given a type which implements `ObjectReferrable`, reads a level name and
  /// a path name and sets one or both on the given object
  ///
  /// NOTE: Mutates the given object
  fn read_object_reference<E: ByteOrder>(&mut self, object: &mut impl ObjectReferrable, map_name: &String) -> Result<()> {
    let level_name = self.read_length_prefixed_string::<E>()?;
    let path_name = self.read_length_prefixed_string::<E>()?;

    if &level_name != map_name {
      object.set_level_name(level_name);
    }
    object.set_path_name(path_name);

    Ok(())
  }

  /// Reads an object of type `Component`'s header
  fn read_component_header<E: ByteOrder>(&mut self, map_name: &String) -> Result<ComponentHeader> {
    let mut component_header = ComponentHeader::default();

    component_header.type_path = self.read_length_prefixed_string::<E>()?;
    self.read_object_reference::<E>(&mut component_header, map_name)?;
    component_header.parent_actor_name = self.read_length_prefixed_string::<E>()?;

    Ok(component_header)
  }

  /// Reads an object of type `Actor`'s header
  fn read_actor_header<E: ByteOrder>(&mut self, map_name: &String) -> Result<ActorHeader> {
    let mut actor_header = ActorHeader::default();

    actor_header.type_path = self.read_length_prefixed_string::<E>()?;
    self.read_object_reference::<E>(&mut actor_header, map_name)?;
    actor_header.needs_transform = self.read_i32::<E>()?;
    actor_header.rotation = self.read_quaternion::<E>()?;
    actor_header.position = self.read_vector::<E>()?;
    actor_header.scale = self.read_vector::<E>()?;
    actor_header.was_placed_in_level = self.read_i32::<E>()?;

    Ok(actor_header)
  }

  /// Reads an object header for a level object by reading a 32-bit integer to
  /// determine the header type and then calling the corresponding function
  fn read_level_object_header<E: ByteOrder>(&mut self, map_name: &String) -> Result<ObjectHeader> {
    let object_type = self.read_i32::<E>()?;
    match ObjectType::from_i32(object_type) {
      Some(ObjectType::Component) => Ok(ObjectHeader::Component(self.read_component_header::<E>(map_name)?)),
      Some(ObjectType::Actor) => Ok(ObjectHeader::Actor(self.read_actor_header::<E>(map_name)?)),
      None => return Err(ParseError::UnknownObject(object_type)),
    }
  }

  /// Reads a byte flag to determine if the following 16 bytes will be a
  /// hex-represented GUID
  fn read_property_guid<E: ByteOrder>(&mut self) -> Result<Option<String>> {
    let has_guid = self.read_u8()?;
    if has_guid == 0 {
      return Ok(None);
    }

    Ok(Some(self.read_hex::<E>(16)?))
  }

  /// Reads a [FicsIt-Network Network trace](https://docs.ficsit.app/ficsit-networks/latest/NetworkTrace.html)
  fn read_fin_network_trace<E: ByteOrder>(&mut self) -> Result<FINNetworkTrace> {
    let mut trace = FINNetworkTrace::default();

    trace.level_name = self.read_length_prefixed_string::<E>()?;
    trace.path_name = self.read_length_prefixed_string::<E>()?;

    let has_prev = self.read_i32::<E>()?;
    if has_prev == 1 {
      trace.prev = Some(Box::new(self.read_fin_network_trace::<E>()?));
    }

    let has_step = self.read_i32::<E>()?;
    if has_step == 1 {
      trace.step = Some(self.read_length_prefixed_string::<E>()?);
    }

    Ok(trace)
  }

  /// Reads a [FicsIt-Network GPUT buffer pixel](https://github.com/Panakotta00/FicsIt-Networks/blob/master/Source/FicsItNetworks/Public/Computer/FINComputerGPUT1.h)
  fn read_fingput1_buffer_pixel<E: ByteOrder>(&mut self) -> Result<FINGPUT1BufferPixel> {
    let mut pixel = FINGPUT1BufferPixel::default();

    pixel.character = self.read_hex::<E>(2)?;
    pixel.foreground_color = self.read_color::<E>()?;
    pixel.background_color = self.read_color::<E>()?;

    Ok(pixel)
  }

  /// Reads a [FicsIt-Network Lua processor state storage](https://github.com/Panakotta00/FicsIt-Networks/blob/master/Source/FicsItNetworksLua/Private/FINLuaProcessorStateStorage.cpp)
  fn read_fin_lua_processor_state_storage<E: ByteOrder>(&mut self, header: &Header, parent_type: Option<&String>) -> Result<FINLuaProcessorStateStorage> {
    let mut data = FINLuaProcessorStateStorage::default();

    let num_traces = self.read_i32::<E>()?;
    for _ in 0..num_traces {
      data.trace.push(self.read_fin_network_trace::<E>()?);
    }

    let num_references = self.read_i32::<E>()?;
    for _ in 0..num_references {
      let mut reference = ObjectReference::default();
      self.read_object_reference::<E>(&mut reference, &header.map_name)?;
      data.reference.push(reference);
    }

    data.thread = self.read_length_prefixed_string::<E>()?;
    data.globals = self.read_length_prefixed_string::<E>()?;

    let num_structs = self.read_i32::<E>()?;
    for _ in 0..num_structs {
      let unk_int_1 = self.read_i32::<E>()?;
      let class_name = self.read_length_prefixed_string::<E>()?;

      if class_name == "/Script/FactoryGame.PrefabSignData" ||
         class_name == "/Script/FicsItNetworks.FINInternetCardHttpRequestFuture" ||
         class_name == "/Script/FactoryGame.InventoryItem" {
        continue;
      }

      let value = match class_name.as_str() {
        "/Script/CoreUObject.Vector" => {
          FINLuaProcessorStateStorageStructValue::Vector(
            self.read_vector::<E>()?,
          )
        },
        "/Script/CoreUObject.LinearColor" => {
          FINLuaProcessorStateStorageStructValue::LinearColor(
            self.read_color::<E>()?,
          )
        },
        "/Script/FactoryGame.InventoryStack" => {
          FINLuaProcessorStateStorageStructValue::InventoryStack(
            InventoryStack {
              unk_str_1: self.read_length_prefixed_string::<E>()?,
              unk_str_2: self.read_length_prefixed_string::<E>()?,
              unk_int_1: self.read_i32::<E>()?,
              unk_int_2: self.read_i32::<E>()?,
              unk_struct_1: self.read_struct_property::<E>(parent_type, header)?,
              unk_str_3: self.read_length_prefixed_string::<E>()?,
            }
          )
        },
        "/Script/FactoryGame.ItemAmount" => {
          FINLuaProcessorStateStorageStructValue::ItemAmount(
            ItemAmount {
              unk_int_1: self.read_i32::<E>()?,
              unk_str_1: self.read_length_prefixed_string::<E>()?,
              unk_int_2: self.read_i32::<E>()?,
            }
          )
        },
        "/Script/FicsItNetworks.FINTrackGraph" => {
          FINLuaProcessorStateStorageStructValue::FINTrackGraph(
            self.read_fin_network_trace::<E>()?,
            self.read_i32::<E>()?,
          )
        },
        "/Script/FicsItNetworks.FINGPUT1Buffer" => {
          let x = self.read_i32::<E>()?;
          let y = self.read_i32::<E>()?;
          let size = self.read_i32::<E>()?;
          let name = self.read_length_prefixed_string::<E>()?;
          let r#type = self.read_length_prefixed_string::<E>()?;
          let length = self.read_i32::<E>()?;
          let mut buffer: Vec<FINGPUT1BufferPixel> = vec![];
          for _ in 0..size {
            buffer.push(self.read_fingput1_buffer_pixel::<E>()?);
          }
          let unk_str_1 = self.read_hex::<E>(45)?;
          FINLuaProcessorStateStorageStructValue::FINGPUT1Buffer(
            FINGPUT1Buffer {
              x,
              y,
              size,
              name,
              r#type,
              length,
              buffer,
              unk_str_1,
            }
          )
        },
        _ => return Err(ParseError::UnknownLuaProcessorStateStorageStructType(class_name)),
      };
      data.structs.push(FINLuaProcessorStateStorageStruct {
        unk_int_1,
        class_name,
        value,
      });
    }

    Ok(data)
  }

  /// Reads an array property whose element is of type struct
  fn read_array_property_struct<E: ByteOrder>(&mut self, num_elements: i32, header: &Header) -> Result<(ArrayPropertyStruct, Vec<ArrayPropertyStructValue>)> {
    let mut struct_meta = ArrayPropertyStruct::default();

    // Always mirrors `property_name` (string)
    self.seek_length_prefixed_string::<E>()?;

    // Always `StructProperty`
    self.seek_length_prefixed_string::<E>()?;

    struct_meta.size_bytes = self.read_i32::<E>()?;

    // Padding
    self.seek_relative(4)?;

    struct_meta.r#type = self.read_length_prefixed_string::<E>()?;
    struct_meta.guid1 = self.read_i32::<E>()?;
    struct_meta.guid2 = self.read_i32::<E>()?;
    struct_meta.guid3 = self.read_i32::<E>()?;
    struct_meta.guid4 = self.read_i32::<E>()?;

    // TODO: What is this?
    self.seek_relative(1)?;

    let mut elements: Vec<ArrayPropertyStructValue> = vec![];

    debug!(">>>>> Reading array property struct of type '{}' with {} elements", struct_meta.r#type, num_elements);

    for _ in 0..num_elements {
      match struct_meta.r#type.as_str() {
        "InventoryItem" => {
          let mut inventory_item = InventoryItem::default();

          inventory_item.unk_int_1 = self.read_i32::<E>()?;
          inventory_item.item_name = self.read_length_prefixed_string::<E>()?;
          inventory_item.level_name = self.read_length_prefixed_string::<E>()?;
          inventory_item.path_name = self.read_length_prefixed_string::<E>()?;

          elements.push(ArrayPropertyStructValue::InventoryItem(inventory_item));
        },
        "Guid" => {
          elements.push(
            ArrayPropertyStructValue::GUID(self.read_hex::<E>(16)?)
          );
        },
        "FINNetworkTrace" => {
          elements.push(
            ArrayPropertyStructValue::FINNetworkTrace(self.read_fin_network_trace::<E>()?)
          );
        },
        "Vector" => {
          elements.push(
            ArrayPropertyStructValue::Vector(self.read_vector_double::<E>()?)
          );
        },
        "LinearColor" => {
          elements.push(
            ArrayPropertyStructValue::LinearColor(self.read_color::<E>()?)
          );
        },
        "FINGPUT1BufferPixel" => {
          elements.push(
            ArrayPropertyStructValue::FINGPUT1BufferPixel(self.read_fingput1_buffer_pixel::<E>()?)
          );
        },
        _ => {
          let mut properties: Vec<Property> = vec![];
          while let Some(p) = self.read_property::<E>(header, Some(&struct_meta.r#type))? {
            debug!(">>>>>> Adding array struct property: {} ({})", p.name, p.r#type);
            properties.push(p);
          }
          debug!(">>>>>> Done reading array struct properties");
          elements.push(ArrayPropertyStructValue::Properties(properties));
        }
      }
    }

    Ok((struct_meta, elements))
  }

  /// Reads an array property
  fn read_array_property<E: ByteOrder>(&mut self, property_name: &String, header: &Header) -> Result<ArrayProperty> {
    let mut property = ArrayProperty::default();

    let r#type = self.read_length_prefixed_string::<E>()?;
    property.r#type = r#type.replace("Property", "");

    // TODO: What is this?
    self.seek_relative(1)?;

    let num_elements = self.read_i32::<E>()?;

    match property.r#type.as_str() {
      "Bool" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Bool(self.read_u8()?))
        }
      },
      "Byte" => {
        if property_name == "mFogOfWarRawData" {
          for _ in 0..(num_elements / 4) {
            self.read_u8()?;
            self.read_u8()?;
            property.elements.push(
              ArrayPropertyValue::Byte(self.read_u8()?)
            );
            self.read_u8()?;
          }
        } else {
          for _ in 0..num_elements {
            property.elements.push(
              ArrayPropertyValue::Byte(self.read_u8()?)
            );
          }
        }
      },
      "Int" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Int(self.read_i32::<E>()?));
        }
      },
      "Int64" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Long(self.read_i64::<E>()?));
        }
      },
      "Float" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Float(self.read_f32::<E>()?));
        }
      },
      "Enum" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Enum(self.read_length_prefixed_string::<E>()?));
        }
      },
      "Str" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Enum(self.read_length_prefixed_string::<E>()?));
        }
      },
      "Text" => {
        for _ in 0..num_elements {
          property.elements.push(ArrayPropertyValue::Text(self.read_text_property::<E>(header.build_version)?));
        }
      },
      "Object" | "Interface" => {
        for _ in 0..num_elements {
          let mut object = ObjectReference::default();
          self.read_object_reference::<E>(&mut object, &header.map_name)?;
          property.elements.push(ArrayPropertyValue::Object(object));
        }
      },
      "SoftObject" => {
        for _ in 0..num_elements {
          let unk_str_1 = self.read_length_prefixed_string::<E>()?;
          let unk_str_2 = self.read_length_prefixed_string::<E>()?;
          let unk_str_3 = self.read_length_prefixed_string::<E>()?;

          debug!("Got 'SoftObject': {}", unk_str_1);
          debug!("Got 'SoftObject': {}", unk_str_2);
          debug!("Got 'SoftObject': {}", unk_str_3);
        }
      },
      "Struct" => {
        let (struct_meta, elements) = self.read_array_property_struct::<E>(num_elements, header)?;
        property.struct_meta = Some(struct_meta);
        for element in elements {
          property.elements.push(ArrayPropertyValue::Struct(element));
        }
      },
      _ => return Err(ParseError::UnknownArrayElementType(property.r#type)),
    }

    Ok(property)
  }

  /// Reads a map property
  fn read_map_property<E: ByteOrder>(&mut self, property_name: &String, parent_type: Option<&String>, header: &Header) -> Result<MapProperty> {
    let parent_type = match parent_type {
      Some(t) => t,
      None => &String::from(""),
    };

    let mut map_property = MapProperty::default();

    let key_type = self.read_length_prefixed_string::<E>()?;
    map_property.key_type = key_type.replace("Property", "");
    let value_type = self.read_length_prefixed_string::<E>()?;
    map_property.value_type = value_type.replace("Property", "");

    // TODO: What is this?
    self.seek_relative(1)?;

    map_property.mode_type = self.read_i32::<E>()?;
    if map_property.mode_type == 2 {
      map_property.unk_mode_2 = Some(self.read_length_prefixed_string::<E>()?);
      map_property.unk_mode_3 = Some(self.read_length_prefixed_string::<E>()?);
    } else if map_property.mode_type == 3 {
      map_property.unk_mode_1 = Some(self.read_hex::<E>(9)?);
      map_property.unk_mode_2 = Some(self.read_length_prefixed_string::<E>()?);
      map_property.unk_mode_3 = Some(self.read_length_prefixed_string::<E>()?);
    }

    let num_pairs = self.read_i32::<E>()?;
    for _ in 0..num_pairs {
      let key = match map_property.key_type.as_str() {
        "Int" => MapPropertyKey::Int(self.read_i32::<E>()?),
        "Int64" => MapPropertyKey::Long(self.read_i64::<E>()?),
        "Name" | "Str" | "Enum" => MapPropertyKey::String(self.read_length_prefixed_string::<E>()?),
        "Object" => {
          let mut object = ObjectReference::default();
          self.read_object_reference::<E>(&mut object, &header.map_name)?;
          MapPropertyKey::Object(object)
        },
        "Struct" => {
          if property_name == "Destroyed_Foliage_Transform" {
            MapPropertyKey::DoubleVector(self.read_vector_double::<E>()?)
          } else if parent_type == "/BuildGunUtilities/BGU_Subsystem.BGU_Subsystem_C" {
            MapPropertyKey::FloatVector(self.read_vector::<E>()?)
          } else if property_name == "mSaveData" || property_name == "mUnresolvedSaveData" {
            MapPropertyKey::IntVector(self.read_vector_int::<E>()?)
          } else {
            let mut keys: Vec<Property> = vec![];
            while let Some(p) = self.read_property::<E>(header, None)? {
              keys.push(p);
            }
            MapPropertyKey::Properties(keys)
          }
        },
        _ => return Err(ParseError::UnknownMapKeyType(map_property.value_type)),
      };

      let value = match map_property.value_type.as_str() {
        "Byte" => {
          if map_property.key_type == "Str" {
            MapPropertyValue::String(self.read_length_prefixed_string::<E>()?)
          } else {
            MapPropertyValue::Byte(self.read_u8()?)
          }
        }
        "Bool" => MapPropertyValue::Bool(self.read_u8()?),
        "Int" => MapPropertyValue::Int(self.read_i32::<E>()?),
        "Int64" => MapPropertyValue::Long(self.read_i64::<E>()?),
        "Float" => MapPropertyValue::Float(self.read_f32::<E>()?),
        "Double" => MapPropertyValue::Double(self.read_f64::<E>()?),
        "Str" => {
          map_property.unk_float_1 = Some(Numeric::Float(self.read_f32::<E>()?));
          map_property.unk_float_2 = Some(Numeric::Float(self.read_f32::<E>()?));
          map_property.unk_float_3 = Some(Numeric::Float(self.read_f32::<E>()?));
          MapPropertyValue::String(self.read_length_prefixed_string::<E>()?)
        },
        "Object" => {
          if parent_type == "/BuildGunUtilities/BGU_Subsystem.BGU_Subsystem_C" {
            map_property.unk_float_1 = Some(Numeric::Float(self.read_f32::<E>()?));
            map_property.unk_float_2 = Some(Numeric::Float(self.read_f32::<E>()?));
            map_property.unk_float_3 = Some(Numeric::Float(self.read_f32::<E>()?));
            map_property.unk_float_4 = Some(Numeric::Float(self.read_f32::<E>()?));
            map_property.unk_str_1 = Some(self.read_length_prefixed_string::<E>()?);
            break;
          } else {
            let mut object = ObjectReference::default();
            self.read_object_reference::<E>(&mut object, &header.map_name)?;
            MapPropertyValue::Object(object)
          }
        },
        "Struct" => {
          if parent_type == "LBBalancerData" {
            map_property.m_normal_index = Some(self.read_i32::<E>()?);
            map_property.m_overflow_index = Some(self.read_i32::<E>()?);
            map_property.m_filter_index = Some(self.read_i32::<E>()?);
            break;
          }

          if parent_type == "/StorageStatsRoom/Sub_SR.Sub_SR_C" {
            map_property.unk_float_1 = Some(Numeric::Double(self.read_f64::<E>()?));
            map_property.unk_float_2 = Some(Numeric::Double(self.read_f64::<E>()?));
            map_property.unk_float_3 = Some(Numeric::Double(self.read_f64::<E>()?));
            break;
          }

          let mut properties: Vec<Property> = vec![];
          while let Some(p) = self.read_property::<E>(header, None)? {
            properties.push(p);
          }
          MapPropertyValue::Struct(properties)
        },

        _ => return Err(ParseError::UnknownMapValueType(map_property.value_type)),
      };

      map_property.keys.push(key);
      map_property.values.push(value);
    }

    Ok(map_property)
  }

  /// Reads a set property
  fn read_set_property<E: ByteOrder>(&mut self, parent_type: Option<&String>, header: &Header) -> Result<SetProperty> {
    let parent_type = match parent_type {
      Some(t) => t,
      None => &String::from(""),
    };

    let mut property = SetProperty::default();

    let r#type = self.read_length_prefixed_string::<E>()?;
    property.r#type = r#type.replace("Property", "");

    // TODO: What is this?
    self.seek_relative(5)?;

    let num_elements = self.read_i32::<E>()?;
    for _ in 0..num_elements {
      let value = match property.r#type.as_str() {
        "Int" => SetPropertyValue::Int(self.read_i32::<E>()?),
        "UInt32" => SetPropertyValue::UInt32(self.read_u32::<E>()?),
        "Name" | "String" => SetPropertyValue::String(self.read_length_prefixed_string::<E>()?),
        "Object" => {
          let mut object = ObjectReference::default();
          self.read_object_reference::<E>(&mut object, &header.map_name)?;
          SetPropertyValue::Object(object)
        },
        "Struct" => {
          if parent_type == "/Script/FactoryGame.FGFoilageRemoval" {
            SetPropertyValue::Vector(self.read_vector::<E>()?)
          } else {
            SetPropertyValue::FINNetworkTrace(self.read_fin_network_trace::<E>()?)
          }
        },
        _ => return Err(ParseError::UnknownSetType(property.r#type)),
      };
      property.values.push(value);
    }

    Ok(property)
  }

  /// Reads a struct property
  fn read_struct_property<E: ByteOrder>(&mut self, parent_type: Option<&String>, header: &Header) -> Result<(String, StructPropertyValue)> {
    let parent_type = match parent_type {
      Some(t) => t,
      None => &String::from(""),
    };

    let r#type = self.read_length_prefixed_string::<E>()?;

    // TODO: What is this?
    self.seek_relative(17)?;

    let value = match r#type.as_str() {
      "Color" => StructPropertyValue::Color(self.read_color_byte()?),
      "LinearColor" => StructPropertyValue::LinearColor(self.read_color::<E>()?),
      "Vector" | "Rotator" => {
        if parent_type == "SpawnData" {
          StructPropertyValue::DoubleVector(self.read_vector_double::<E>()?)
        } else {
          StructPropertyValue::FloatVector(self.read_vector::<E>()?)
        }
      },
      "Vector2D" => StructPropertyValue::DoubleVector2D(self.read_vector2d_double::<E>()?),
      "IntVector4" => StructPropertyValue::IntVector4(self.read_vector4_int::<E>()?),
      "Quat" => StructPropertyValue::DoubleQuaternion(self.read_quaternion_double::<E>()?),
      "Vector4" => StructPropertyValue::DoubleVector4(self.read_vector4_double::<E>()?),
      "Box" => StructPropertyValue::Box(StructPropertyBox {
        min: self.read_vector_double::<E>()?,
        max: self.read_vector_double::<E>()?,
        is_valid: self.read_u8()?,
      }),
      "RailroadTrackPosition" => {
        let mut object = ObjectReference::default();
        self.read_object_reference::<E>(&mut object, &header.map_name)?;
        StructPropertyValue::RailroadTrackPosition(
          StructPropertyRailroadTrackPosition {
            object,
            offset: self.read_f32::<E>()?,
            forward: self.read_f32::<E>()?,
          }
        )
      },
      "TimerHandle" => StructPropertyValue::TimerHandle(self.read_length_prefixed_string::<E>()?),
      "Guid" => StructPropertyValue::GUID(self.read_hex::<E>(16)?),
      "InventoryItem" => {
        let unk_int_1 = self.read_i32::<E>()?;
        let item_name = self.read_length_prefixed_string::<E>()?;
        let mut object = ObjectReference::default();
        self.read_object_reference::<E>(&mut object, &header.map_name)?;
        let property = match self.read_property::<E>(header, None)? {
          Some(p) => p,
          None => return Err(ParseError::MissingInventoryItemProperty(item_name)),
        };
        StructPropertyValue::InventoryItem(StructPropertyInventoryItem {
          unk_int_1,
          item_name,
          object,
          property: Box::new(property),
        })
      },
      "FluidBox" => StructPropertyValue::FluidBox(self.read_f32::<E>()?),
      "SlateBrush" => StructPropertyValue::SlateBrush(self.read_length_prefixed_string::<E>()?),
      "DateTime" => StructPropertyValue::DateTime(self.read_i64::<E>()?),
      "FINNetworkTrace" => StructPropertyValue::FINNetworkTrace(self.read_fin_network_trace::<E>()?),
      "FINLuaProcessorStateStorage" => {
        StructPropertyValue::FINLuaProcessorStateStorage(self.read_fin_lua_processor_state_storage::<E>(header, None)?)
      },
      "FICFrameRange" => StructPropertyValue::FICFrameRange(FrameRange {
        begin: self.read_i64::<E>()?,
        end: self.read_i64::<E>()?,
      }),
      "IntPoint" => StructPropertyValue::IntVector2D(self.read_vector2d_int::<E>()?),
      _ => {
        let mut properties: Vec<Property> = vec![];
        while let Some(p) = self.read_property::<E>(header, Some(&r#type))? {
          properties.push(p);
        }
        StructPropertyValue::Properties(properties)
      },
    };

    Ok((r#type, value))
  }

  /// Reads a text property
  fn read_text_property<E: ByteOrder>(&mut self, build_version: i32) -> Result<TextProperty> {
    let mut property = TextProperty::default();

    property.flags = self.read_i32::<E>()?;
    property.history_type = self.read_u8()?;

    match property.history_type {
      0 => {
        let mut history = BaseHistory::default();
        history.namespace = self.read_length_prefixed_string::<E>()?;
        history.key = self.read_length_prefixed_string::<E>()?;
        history.value = self.read_length_prefixed_string::<E>()?;
        property.value = TextPropertyHistory::BaseHistory(history);
      },
      1 | 3 => {
        let mut history = ArgumentHistory::default();
        history.source_format = Box::new(self.read_text_property::<E>(build_version)?);
        history.num_arguments = self.read_i32::<E>()?;
        for _ in 0..history.num_arguments {
          let mut argument = Argument::default();
          argument.name = self.read_length_prefixed_string::<E>()?;
          argument.value_type = self.read_u8()?;
          match argument.value_type {
            4 => {
              argument.value = Box::new(self.read_text_property::<E>(build_version)?);
            },
            _ => return Err(ParseError::UnknownTextArgumentValueType(argument.value_type)),
          }
        }
      },
      10 => {
        let mut history = TransformHistory::default();
        history.source_text = Box::new(self.read_text_property::<E>(build_version)?);
        history.transform_type = self.read_u8()?;
        property.value = TextPropertyHistory::TransformHistory(history);
      },
      11 => {
        let mut history = StringTableEntryHistory::default();
        history.table_id = self.read_length_prefixed_string::<E>()?;
        history.text_key = self.read_length_prefixed_string::<E>()?;
        property.value = TextPropertyHistory::StringTableEntryHistory(history);
      },
      255 => {
        let mut history = NoneHistory::default();
        history.has_culture_invariant_string = self.read_i32::<E>()?;
        history.value = self.read_length_prefixed_string::<E>()?;
        property.value = TextPropertyHistory::NoneHistory(history);
      },
      _ => return Err(ParseError::UnknownTextHistoryType(property.history_type)),
    }

    Ok(property)
  }

  /// Reads a property
  fn read_property<E: ByteOrder>(&mut self, header: &Header, parent_type: Option<&String>) -> Result<Option<Property>> {
    let name = self.read_length_prefixed_string::<E>()?;
    if name == "None" {
      return Ok(None);
    }

    let name = name;

    // TODO: What is this?
    let extra_byte = self.read_u8()?;
    if extra_byte != 0 {
      self.seek_relative(-1)?;
    }

    let r#type = self.read_length_prefixed_string::<E>()?;

    debug!(">>>>> Reading property '{}' for '{:?}", name, parent_type);

    // Most/all properties end in "Property" e.g. "ObjectProperty" and we'd like to
    // remove the redundancy
    let r#type = r#type.replace("Property", "");

    let size = self.read_i32::<E>()?;
    let index = self.read_i32::<E>()?;

    let mut value = match PropertyValue::from_str(&r#type) {
      Ok(p) => p,
      Err(_) => return Err(ParseError::UnknownPropertyType(r#type)),
    };

    let mut guid: Option<String> = None;
    match &mut value {
      PropertyValue::Array(p) => {
        *p = self.read_array_property::<E>(&name, header)?;
      },
      PropertyValue::Bool(p) => {
        *p = self.read_u8()?;
        guid = self.read_property_guid::<E>()?;
      },
      PropertyValue::Byte(p) => {
        let mut byte_property = ByteProperty::default();
        byte_property.r#type = self.read_length_prefixed_string::<E>()?;
        if byte_property.r#type == "None" {
          byte_property.byte_value = Some(self.read_u8()?);
        } else {
          byte_property.string_value = Some(self.read_length_prefixed_string::<E>()?);
        }
        *p = byte_property;
      },
      PropertyValue::Double(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_f64::<E>()?;
      },
      PropertyValue::Enum(p) => {
        let mut enum_property = HashMap::new();
        let name = self.read_length_prefixed_string::<E>()?;
        guid = self.read_property_guid::<E>()?;
        enum_property.insert(name, self.read_length_prefixed_string::<E>()?);
        *p = enum_property;
      },
      PropertyValue::Float(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_f32::<E>()?;
      }
      PropertyValue::Int(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_i32::<E>()?;
      },
      PropertyValue::Int8(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_i8()?;
      },
      PropertyValue::Int64(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_i64::<E>()?;
      },
      PropertyValue::Map(p) => {
        *p = self.read_map_property::<E>(&name, parent_type, &header)?;
      },
      PropertyValue::Object(p) => {
        guid = self.read_property_guid::<E>()?;
        let mut object = ObjectReference::default();
        self.read_object_reference::<E>(&mut object, &header.map_name)?;
        *p = object;
      },
      PropertyValue::Set(p) => {
        *p = self.read_set_property::<E>(parent_type, &header)?;
      },
      PropertyValue::String(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_length_prefixed_string::<E>()?;
      },
      PropertyValue::Struct(p) => {
        *p = self.read_struct_property::<E>(parent_type, &header)?;
      },
      PropertyValue::Text(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_text_property::<E>(header.build_version)?;
      },
      PropertyValue::UInt32(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_u32::<E>()?;
      },
      PropertyValue::UInt64(p) => {
        guid = self.read_property_guid::<E>()?;
        *p = self.read_u64::<E>()?;
      },
    }

    Ok(Some(Property {
      name,
      size,
      r#type,
      index,
      guid,
      value,
    }))
  }

  /// Reads an object by reading its meta followed by its properties and
  /// validating its supposed size against actual size and seeking past any gap
  fn read_object<E: ByteOrder>(&mut self, object_header: &ObjectHeader, header: &Header) -> Result<Object> {
    let mut object = match object_header {
      ObjectHeader::Actor(_) => Object::Actor(ActorObject::default()),
      ObjectHeader::Component(_) => Object::Component(ComponentObject::default()),
    };

    let object_save_version = self.read_i32::<E>()?;
    object.set_save_version(object_save_version);

    // TODO: What is this?
    self.seek_relative(4)?;

    let object_size_bytes = self.read_i32::<E>()?;

    // Marks the start byte so that, after reading the object's components and
    // properties (when the object has ended), the position can be checked to
    // see if any bytes are missing
    let start_byte = self.stream_position()?;

    object.set_size_bytes(object_size_bytes);

    if let Object::Actor(ref mut object) = object {
      self.read_object_reference::<E>(object, &header.map_name)?;

      object.num_components = self.read_i32::<E>()?;
      debug!(">>>> Reading {} object components", object.num_components);
      for _ in 0..object.num_components {
        let mut component = ObjectReference::default();
        self.read_object_reference::<E>(&mut component, &header.map_name)?;
        object.components.push(component);
      }
    }

    let current_position = self.stream_position()?;
    if current_position - start_byte == object_size_bytes as u64 {
      object.set_should_be_nulled();
      return Ok(object);
    }

    debug!(">>>> Reading object properties");
    while let Some(property) = self.read_property::<E>(header, Some(object_header.get_type_path()))? {
      debug!(">>>> Adding object property: {}", property.name);
      object.add_property(property);
    }

    let current_position = self.stream_position()?;

    // By adding the given size of this object to the start byte, it can be
    // determined where this object *should* end and whether or not the object
    // ends where it should have
    let current_object_end_position = start_byte + object_size_bytes as u64;

    // Pre-empts a negative value for `missing_bytes` by first checking if
    // the supposed end position is *beyond* the current position. In other
    // words, if the current object is *larger* than was told
    if current_object_end_position < current_position {
      return Err(ParseError::ObjectLength(object_header.get_type_path().clone()))
    }

    // Handles any number of missing bytes by seeking past that amount of bytes
    let missing_bytes = current_object_end_position - current_position;
    if missing_bytes > 4 {
      if object_header.get_type_path().starts_with("/Script/FactoryGame.FG") {
        self.seek_relative(8)?;
      } else {
        let skipped = self.read_hex::<E>(missing_bytes as usize)?;
        warn!("Missing {missing_bytes} bytes at {}: {skipped}", object_header.get_type_path());
      }
    } else {
      self.seek_relative(4)?;
    }

    Ok(object)
  }

  /// Reads a single level by reading its name, object headers, collectables,
  /// objects, and seeking past a repeated set of collectables if set
  fn read_level<E: ByteOrder>(&mut self, level_index: i32, is_last_level: bool, header: &Header) -> Result<Level> {
    let mut level = Level::default();

    level.name = if is_last_level {
      format!("Level {}", header.map_name)
    } else {
      self.read_length_prefixed_string::<E>()?
    };
    debug!(">> Level name: '{}'", level.name);

    let object_headers_and_collectables_size_bytes = self.read_i64::<E>()?;
    let level_start_byte = self.stream_position()? as i64;

    // Reads object headers for this level
    let num_object_headers = self.read_i32::<E>()?;
    debug!(">>> Reading {} level object headers", num_object_headers);
    for _i in 0..num_object_headers {
      level.object_headers.push(self.read_level_object_header::<E>(&header.map_name)?);
    }

    // Reads collectables for this level
    let current_position = self.stream_position()? as i64;
    let stop_byte = level_start_byte + object_headers_and_collectables_size_bytes - 4;
    if current_position < stop_byte {
      let num_collectables = self.read_i32::<E>()?;
      debug!(">>> Reading {} level collectables", num_collectables);
      for _ in 0..num_collectables {
        let mut collectable = Collectable::default();
        self.read_object_reference::<E>(&mut collectable, &header.map_name)?;
        level.collectables.push(collectable);
      }
    } else if current_position == stop_byte {
      debug!(">>> No collectables to read");
      // TODO: What is this?
      self.seek_relative(4)?;
    }

    // Represents the size of this level's objects in bytes (as i64)
    self.seek_relative(8)?;

    // Reads objects for this level
    let num_objects = self.read_i32::<E>()?;
    debug!(">>> Reading {} level objects", num_objects);
    for i in 0..num_objects {
      debug!(">>>> Reading level object {}/{}", i + 1, num_objects);

      let object_header = match level.object_headers.get(i as usize) {
        Some(o) => o,
        None => return Err(ParseError::MissingObjectHeader(level.name)),
      };
      let object = self.read_object::<E>(object_header, header)?;
      debug!("Level {}, Object {}/{}: {:#?}", level_index, i + 1, num_objects, object);
      level.objects.push(object);
    }

    // Collectables are repeated after the object list so these can be
    // safely skipped
    let num_second_collectables = self.read_i32::<E>()?;
    for _ in 0..num_second_collectables {
      self.seek_length_prefixed_string::<E>()?;
      self.seek_length_prefixed_string::<E>()?;
    }

    Ok(level)
  }

  /// Reads a 32-bit integer to determine the number of levels present and then
  /// reads each level one-by-one
  fn read_levels<E: ByteOrder>(&mut self, header: &Header) -> Result<Vec<Level>> {
    let mut levels: Vec<Level> = vec![];

    let num_levels = self.read_i32::<E>()?;
    debug!(">> Reading {num_levels} levels");
    for i in 0..num_levels {
      debug!(">> Reading level {}/{} @ byte {}", i + 1, num_levels, self.stream_position()?);
      levels.push(self.read_level::<E>(i, i == num_levels, header)?);
    }

    Ok(levels)
  }
}

/// Auto-implements the above trait for all types which also implement both
/// `io::Read` and `io::Seek`
impl<R: io::Read + io::Seek> ReadSaveFileBytes for R {}
