use core::str;
use std::error::Error;
use std::fs;
use std::io::{self, Read, Seek};
use std::result;
use std::str::FromStr;

use byteorder::{ByteOrder, LittleEndian, ReadBytesExt};
use env_logger::{self, Env};
use flate2::bufread::ZlibDecoder;
use log::{debug, info, warn};

mod property;
mod save;

use crate::property::*;
use crate::save::*;

pub type Result<T> = result::Result<T, Box<dyn Error>>;

const MIN_SAVE_FILE_VERSION: i32 = 41;
const FILE_PATH: &str = "C:\\Users\\devin\\AppData\\Local\\FactoryGame\\Saved\\SaveGames\\76561198025332073\\Derp Quad_200724-013919.sav";

pub trait ReadSaveFileBytes: ReadBytesExt + Seek {
  fn read_length_prefixed_string<E: ByteOrder>(&mut self) -> Result<String> {
    let len = self.read_i32::<E>()?;

    let mut string: String;
    // If `len` is positive, it's UTF-8-encoded
    // If `len` is negative, it's UTF-16-encoded
    // Otherwise, it's an empty string
    if len > 0 {
      let mut dst: Vec<u8> = vec![0; len as usize];
      self.read_exact(&mut dst)?;
      string = String::from_utf8(dst)?;
  
      // Removes the null termination byte
      string.pop();
    } else if len < 0 {
      let mut dst: Vec<u16> = vec![0; len.abs() as usize];
      self.read_u16_into::<E>(&mut dst)?;
      string = String::from_utf16(&dst[..])?;
      
      // Removes the null termination byte
      string.pop();
    } else {
      string = String::from("");
    }
  
    Ok(string)
  }

  fn seek_length_prefixed_string<E: ByteOrder>(&mut self) -> Result<()> {
    let len = self.read_i32::<E>()?;
    self.seek_relative(len as i64)?;
    Ok(())
  }

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

    let mut hash: Vec<u16> = vec![0; 10];
    self.read_u16_into::<E>(&mut hash)?;
    header.saved_data_hash = String::from_utf16(&hash)?;
    header.is_creative_mode_enabled = self.read_i32::<E>()?;

    Ok(header)
  }

  fn read_chunk<E: ByteOrder>(&mut self) -> Result<Vec<u8>> {
    let unreal_engine_package_sig = self.read_u32::<E>()?;
    let unreal_engine_package_sig = format!("{:X}", unreal_engine_package_sig);
    assert!(unreal_engine_package_sig == "9E2A83C1");

    // Padding
    self.seek_relative(4)?;

    let max_chunk_size = self.read_u32::<E>()?;
    assert!(max_chunk_size == 131_072);

    // Padding
    self.seek_relative(5)?;
    
    let current_chunk_size = self.read_u32::<E>()?;

    // Padding
    self.seek_relative(28)?;

    let mut chunk_bytes: Vec<u8> = vec![0; current_chunk_size as usize];
    self.read_exact(&mut chunk_bytes)?;
    
    Ok(chunk_bytes)
  }

  fn read_chunks<E: ByteOrder>(&mut self, stop_byte: u64) -> Result<Vec<Vec<u8>>> {
    let mut chunks: Vec<Vec<u8>> = vec![];
  
    while self.stream_position()? < stop_byte {
      let chunk_bytes = self.read_chunk::<E>()?;
      chunks.push(chunk_bytes);
    }
  
    Ok(chunks)
  }

  fn read_partitions<E: ByteOrder>(&mut self) -> Result<Partitions> {
    let mut partitions = Partitions::default();
    let num_partitions = self.read_i32::<E>()?;
    partitions.unk_str_1 = self.read_length_prefixed_string::<E>()?;
    partitions.unk_num_1 = self.read_i64::<E>()?;
    partitions.unk_num_2 = self.read_i32::<E>()?;
    partitions.unk_str_2 = self.read_length_prefixed_string::<E>()?;
    partitions.unk_num_3 = self.read_i32::<E>()?;
  
    for _ in 1..num_partitions {
      let key = self.read_length_prefixed_string::<E>().unwrap();

      let mut partition = Partition::default();
      partition.unk_num_1 = self.read_i32::<E>()?;
      partition.unk_num_2 = self.read_i32::<E>()?;
  
      let num_levels = self.read_i32::<E>().unwrap();
      for _ in 0..num_levels {
        let level_key = self.read_length_prefixed_string::<E>().unwrap();
        let level_value = self.read_u32::<E>().unwrap();
        partition.levels.insert(level_key, level_value);
      }

      partitions.partitions.insert(key, Partition::default());
    }
    Ok(partitions)
  }

  fn read_object_reference<E: ByteOrder>(&mut self, object: &mut impl ObjectReferrable, map_name: &String) -> Result<()> {
    let level_name = self.read_length_prefixed_string::<E>()?;
    let path_name = self.read_length_prefixed_string::<E>()?;

    if &level_name != map_name {
      object.set_level_name(level_name);
    }
    object.set_path_name(path_name);

    Ok(())
  }

  fn read_component_header<E: ByteOrder>(&mut self, map_name: &String) -> Result<ComponentHeader> {
    let mut component_header = ComponentHeader::default();

    component_header.type_path = self.read_length_prefixed_string::<E>()?;
    self.read_object_reference::<E>(&mut component_header, map_name)?;
    component_header.parent_actor_name = self.read_length_prefixed_string::<E>()?;

    Ok(component_header)
  }

  fn read_quaternion<E: ByteOrder>(&mut self) -> Result<Quaternion> {
    Ok(Quaternion {
      x: self.read_f32::<E>()?,
      y: self.read_f32::<E>()?,
      z: self.read_f32::<E>()?,
      w: self.read_f32::<E>()?,
    })
  }

  fn read_vector<E: ByteOrder>(&mut self) -> Result<Vector> {
    Ok(Vector {
      x: self.read_f32::<E>()?,
      y: self.read_f32::<E>()?,
      z: self.read_f32::<E>()?,
    })
  }

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

  fn read_level_object_header<E: ByteOrder>(&mut self, map_name: &String) -> Result<ObjectHeader> {
    let object_type = self.read_i32::<E>()?;
    match ObjectType::from_i32(object_type) {
      Some(ObjectType::Component) => Ok(ObjectHeader::Component(self.read_component_header::<E>(map_name)?)),
      Some(ObjectType::Actor) => Ok(ObjectHeader::Actor(self.read_actor_header::<E>(map_name)?)),
      None => panic!("Unknown object type: {object_type}"),
    }
  }

  fn read_property_guid<E: ByteOrder>(&mut self) -> Result<Option<String>> {
    let has_guid = self.read_u8()?;
    if has_guid == 0 {
      return Ok(None);
    }

    let mut guid: Vec<u16> = vec![0; 16];
    self.read_u16_into::<E>(&mut guid)?;
    Ok(Some(String::from_utf16(&guid)?))
  }

  fn read_property<E: ByteOrder>(&mut self) -> Result<Option<Property>> {
    let mut property = Property::default();

    property.name = self.read_length_prefixed_string::<E>()?;
    if property.name == "None" {
      return Ok(None);
    }

    // TODO: What is this?
    let extra_byte = self.read_u8()?;
    if extra_byte != 0 {
      self.seek_relative(-1)?;
    }

    let r#type = self.read_length_prefixed_string::<E>()?;
    // Most/all properties end in "Property" e.g. "ObjectProperty" and we'd like to
    // remove the redundancy
    property.r#type = r#type.replace("Property", "");

    property.size = self.read_i32::<E>()?;
    property.index = self.read_i32::<E>()?;

    let mut value = match PropertyValue::from_str(&property.r#type) {
      Ok(p) => p,
      Err(e) => panic!("Unknown property '{}' encounted: {}", r#type, e),
    };

    match &mut value {
      PropertyValue::Int(p) => {
        property.guid = self.read_property_guid::<E>()?;
        *p = self.read_i32::<E>()?;
      },
      // This should never be encounted; The "None" variant only exists as a
      // placeholder so we can create a default `Property``
      PropertyValue::None => panic!(),
      _ => {},
    }

    property.value = value;

    debug!(">>>> Property: {:?}", property);

    Ok(Some(property))
  }

  fn read_object<E: ByteOrder>(&mut self, object_header: &ObjectHeader, header: &Header) -> Result<Object> {
    let mut object = match object_header {
      ObjectHeader::Actor(_) => Object::Actor(ActorObject::default()),
      ObjectHeader::Component(_) => Object::Component(ComponentObject::default()),
    };

    object.set_save_version(self.read_i32::<E>()?);
    let start_byte = self.stream_position()?;

    // TODO: What is this?
    self.seek_relative(4)?;

    let object_size_bytes = self.read_i32::<E>()?;
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

    while let Some(property) = self.read_property::<E>()? {
      object.add_property(property);
    }

    let current_position = self.stream_position()?;
    let missing_bytes = current_position - (start_byte + object_size_bytes as u64);
    if missing_bytes > 4 {
      warn!("Missing {missing_bytes} bytes at {}", object_header.get_type_path());
    } else {
      self.seek_relative(4)?;
    }

    Ok(object)
  }

  fn read_level<E: ByteOrder>(&mut self, is_last_level: bool, header: &Header) -> Result<Level> {
    let mut level = Level::default();

    level.name = if is_last_level {
      format!("Level {}", header.map_name)
    } else {
      self.read_length_prefixed_string::<E>()?
    };
    debug!(">> Reading level: {}", level.name);

    level.object_headers_and_collectables_size_bytes = self.read_i64::<E>()?;
    let level_start_byte = self.stream_position()? as i64;

    // Reads object headers for this level
    level.num_object_headers = self.read_i32::<E>()?;
    debug!(">>> Reading {} level object headers", level.num_object_headers);
    for _i in 0..level.num_object_headers {
      level.object_headers.push(self.read_level_object_header::<E>(&header.map_name)?);
    }

    // Reads collectables for this level
    let current_position = self.stream_position()? as i64;
    let stop_byte = level_start_byte + level.object_headers_and_collectables_size_bytes - 4;
    if current_position < stop_byte {
      level.num_collectables = self.read_i32::<E>()?;
      debug!(">>> Reading {} level collectables", level.num_collectables);
      for _ in 0..level.num_collectables {
        let mut collectable = Collectable::default();
        self.read_object_reference::<E>(&mut collectable, &header.map_name)?;
        level.collectables.push(collectable);
      }
    } else if current_position == stop_byte {
      debug!(">>> No collectables to read");
      // TODO: What is this?
      self.seek_relative(4)?;
    }

    // Reads objects for this level
    level.objects_size_bytes = self.read_i64::<E>()?;
    level.num_objects = self.read_i32::<E>()?;
    debug!(">>> Reading {} level objects", level.num_objects);
    for i in 0..level.num_objects {
      debug!(">>>> Reading level object {i}");

      let object_header = match level.object_headers.get(i as usize) {
        Some(o) => o,
        None => panic!("No header for object at index {i}"),
      };
      level.objects.push(self.read_object::<E>(object_header, header)?);
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

  fn read_levels<E: ByteOrder>(&mut self, header: &Header) -> Result<Vec<Level>> {
    let mut levels: Vec<Level> = vec![];

    let num_levels = self.read_i32::<E>()?;
    debug!(">> Reading {num_levels} levels");
    for i in 0..=num_levels {
      levels.push(self.read_level::<E>(i == num_levels, header)?);
    }

    Ok(levels)
  }
}

impl<R: io::Read + ?Sized + io::Seek> ReadSaveFileBytes for R {}

fn main() {
  info!("Satisfactory Save File Reader");

  // Setups up the logger
  let env = Env::default();
  env_logger::init_from_env(env);

  let file_bytes = match fs::read(FILE_PATH) {
    Ok(b) => b,
    Err(e) => panic!("Error reading file ({FILE_PATH}): {:?}", e),
  };

  let file_size_bytes = file_bytes.len();
  
  let mut cursor = io::Cursor::new(file_bytes);

  info!("> Reading file header");
  let header = match cursor.read_header::<LittleEndian>() {
    Ok(h) => h,
    Err(e) => panic!("Error parsing save file header: {}", e),
  };
  assert!(header.save_file_version > MIN_SAVE_FILE_VERSION);
  debug!("Header: {:#?}", header);

  info!("> Reading chunk metadata");
  let chunks = match cursor.read_chunks::<LittleEndian>(file_size_bytes as u64) {
    Ok(c) => c,
    Err(e) => panic!("Error parsing chunk meta: {}", e),
  };
  drop(cursor);

  info!("> Decompressing chunk bytes");
  let mut body_bytes: Vec<u8> = vec![];
  for chunk_bytes in chunks {
    let mut z = ZlibDecoder::new(&chunk_bytes[..]);
    z.read_to_end(&mut body_bytes).unwrap();
  }

  let mut body_cursor = io::Cursor::new(body_bytes);
  // Skips some unknown data
  body_cursor.seek_relative(8).unwrap();

  info!("> Reading partition data");
  let partitions = match body_cursor.read_partitions::<LittleEndian>() {
    Ok(p) => p,
    Err(e) => panic!("Error reading partition data: {}", e),
  };
  debug!("{}", serde_json::to_string_pretty(&partitions).unwrap());

  info!("> Reading level data");
  let _levels = match body_cursor.read_levels::<LittleEndian>(&header) {
    Ok(l) => l,
    Err(e) => panic!("Error reading level data: {}", e),
  };

  info!("> Success!");
}
