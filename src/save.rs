use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::property::*;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Header {
  pub save_header_version: i32,
  pub save_file_version: i32,
  pub build_version: i32,
  pub map_name: String,
  pub map_options: String,
  pub session_name: String,
  pub played_seconds: i32,
  pub save_timestamp: i64,
  pub session_visibility: i8,
  pub editor_object_version: i32,
  pub mod_metadata: String,
  pub mod_flags: i32,
  pub save_identifier: String,
  pub is_partitioned_world: i32,
  pub saved_data_hash: String,
  pub is_creative_mode_enabled: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Partition {
  pub unk_num_1: i32,
  pub unk_num_2: i32,
  pub levels: HashMap<String, u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Partitions {
  pub unk_str_1: String,
  pub unk_str_2: String,
  pub unk_num_1: i64,
  pub unk_num_2: i32,
  pub unk_num_3: i32,
  pub partitions: HashMap<String, Partition>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Level {
  pub name: String,
  pub object_headers_and_collectables_size_bytes: i64,
  pub num_object_headers: i32,
  pub object_headers: Vec<ObjectHeader>,
  pub num_collectables: i32,
  pub collectables: Vec<Collectable>,
  pub objects_size_bytes: i64,
  pub num_objects: i32,
  pub objects: Vec<Object>,
}


pub enum ObjectType {
  Component,
  Actor,
}
  
impl ObjectType {
  pub fn from_i32(value: i32) -> Option<ObjectType> {
    match value {
      0 => Some(ObjectType::Component),
      1 => Some(ObjectType::Actor),
      _ => None,
    }
  }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ObjectHeader {
  Component(ComponentHeader),
  Actor(ActorHeader),
}

impl ObjectHeader {
  pub fn get_type_path(&self) -> &String {
    match self {
      ObjectHeader::Component(c) => &c.type_path,
      ObjectHeader::Actor(a) => &a.type_path,
    }
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentHeader {
  pub type_path: String,
  pub root_object: Option<String>,
  pub instance_name: String,
  pub parent_actor_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Quaternion {
  pub x: f32,
  pub y: f32,
  pub z: f32,
  pub w: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vector {
  pub x: f32,
  pub y: f32,
  pub z: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorHeader {
  pub type_path: String,
  pub root_object: Option<String>,
  pub instance_name: String,
  pub needs_transform: i32,
  pub rotation: Quaternion,
  pub position: Vector,
  pub scale: Vector,
  pub was_placed_in_level: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentObject {
  pub should_be_nulled: bool, // Virtual property
  pub save_version: i32,
  pub size_bytes: i32,
  pub properties: Vec<Property>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorObject {
  pub should_be_nulled: bool, // Virtual property
  pub save_version: i32,
  pub size_bytes: i32,
  pub parent_object_root: String,
  pub parent_object_name: String,
  pub num_components: i32,
  pub components: Vec<ObjectReference>,
  pub properties: Vec<Property>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Object {
  Component(ComponentObject),
  Actor(ActorObject),
}

impl Object {
  pub fn set_save_version(&mut self, save_version: i32) {
    match self {
      Object::Actor(a) => a.save_version = save_version,
      Object::Component(c) => c.save_version = save_version,
    }
  }

  pub fn set_size_bytes(&mut self, size_bytes: i32) {
    match self {
      Object::Actor(a) => a.size_bytes = size_bytes,
      Object::Component(c) => c.size_bytes = size_bytes,
    }
  }

  pub fn set_should_be_nulled(&mut self) {
    match self {
      Object::Actor(a) => a.should_be_nulled = true,
      Object::Component(c) => c.should_be_nulled = true,
    }
  }

  pub fn add_property(&mut self, property: Property) {
    match self {
      Object::Actor(a) => a.properties.push(property),
      Object::Component(c) => c.properties.push(property),
    }
  }
}

// This is the same as a Collectable but 
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ObjectReference {
  pub level_name: String,
  pub path_name: String,
}

pub type Collectable = ObjectReference;

pub trait ObjectReferrable {
  fn set_level_name(&mut self, level_name: String);
  fn set_path_name(&mut self, path_name: String);
}

impl ObjectReferrable for ObjectReference {
  fn set_level_name(&mut self, level_name: String) {
    self.level_name = level_name;
  }

  fn set_path_name(&mut self, path_name: String) {
    self.path_name = path_name;
  }
}

impl ObjectReferrable for ComponentHeader {
  fn set_level_name(&mut self, level_name: String) {
    self.root_object = Some(level_name);
  }
  
  fn set_path_name(&mut self, path_name: String) {
    self.instance_name = path_name;
  }
}

impl ObjectReferrable for ActorHeader {
  fn set_level_name(&mut self, level_name: String) {
    self.root_object = Some(level_name);
  }
  
  fn set_path_name(&mut self, path_name: String) {
    self.instance_name = path_name;
  }
}

impl ObjectReferrable for ActorObject {
  fn set_level_name(&mut self, level_name: String) {
    self.parent_object_root = level_name;
  }

  fn set_path_name(&mut self, path_name: String) {
    self.parent_object_name = path_name;
  }
}
