use std::collections::HashMap;

use serde::{Serialize, Deserialize};

use crate::property::*;

const GAME_PATHS: [&str; 2] = [
  "/Game/FactoryGame/-Shared/Blueprint/BP_GameState.BP_GameState_C",
  "/Game/FactoryGame/-Shared/Blueprint/BP_GameMode.BP_GameMode_C",
];
const PLAYER_STATE_PATHS: [&str; 1] = [
  "/Game/FactoryGame/Character/Player/BP_PlayerState.BP_PlayerState_C",
];
const CONVEYOR_PATHS: [&str; 10] = [
  "/Game/FactoryGame/Buildable/Factory/ConveyorBeltMk1/Build_ConveyorBeltMk1.Build_ConveyorBeltMk1_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorBeltMk2/Build_ConveyorBeltMk2.Build_ConveyorBeltMk2_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorBeltMk3/Build_ConveyorBeltMk3.Build_ConveyorBeltMk3_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorBeltMk4/Build_ConveyorBeltMk4.Build_ConveyorBeltMk4_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorBeltMk5/Build_ConveyorBeltMk5.Build_ConveyorBeltMk5_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorLiftMk1/Build_ConveyorLiftMk1.Build_ConveyorLiftMk1_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorLiftMk2/Build_ConveyorLiftMk2.Build_ConveyorLiftMk2_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorLiftMk3/Build_ConveyorLiftMk3.Build_ConveyorLiftMk3_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorLiftMk4/Build_ConveyorLiftMk4.Build_ConveyorLiftMk4_C",
  "/Game/FactoryGame/Buildable/Factory/ConveyorLiftMk5/Build_ConveyorLiftMk5.Build_ConveyorLiftMk5_C",
];
const POWER_LINE_PATHS: [&str; 2] = [
  "/Game/FactoryGame/Buildable/Factory/PowerLine/Build_PowerLine.Build_PowerLine_C",
  "/Game/FactoryGame/Events/Christmas/Buildings/PowerLineLights/Build_XmassLightsLine.Build_XmassLightsLine_C",
];
const DRONE_TRANSPORT_PATHS: [&str; 1] = [
  "/Game/FactoryGame/Buildable/Factory/DroneStation/BP_DroneTransport.BP_DroneTransport_C",
];
const CIRCUIT_PATHS: [&str; 1] = [
  "/Game/FactoryGame/-Shared/Blueprint/BP_CircuitSubsystem.BP_CircuitSubsystem_C",
];
const VEHICLE_PATHS: [&str; 6] = [
  "/Game/FactoryGame/Buildable/Vehicle/Tractor/BP_Tractor.BP_Tractor_C",
  "/Game/FactoryGame/Buildable/Vehicle/Truck/BP_Truck.BP_Truck_C",
  "/Game/FactoryGame/Buildable/Vehicle/Explorer/BP_Explorer.BP_Explorer_C",
  "/Game/FactoryGame/Buildable/Vehicle/Cyberwagon/Testa_BP_WB.Testa_BP_WB_C",
  "/Game/FactoryGame/Buildable/Vehicle/Golfcart/BP_Golfcart.BP_Golfcart_C",
  "/Game/FactoryGame/Buildable/Vehicle/Golfcart/BP_GolfcartGold.BP_GolfcartGold_C",
];
const LOCOMOTIVE_PATHS: [&str; 1] = [
  "/Game/FactoryGame/Buildable/Vehicle/Train/Locomotive/BP_Locomotive.BP_Locomotive_C",
];
const FREIGHT_WAGON_PATHS: [&str; 1] = [
  "/Game/FactoryGame/Buildable/Vehicle/Train/Wagon/BP_FreightWagon.BP_FreightWagon_C",
];

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Save {
  pub header: Header,
  pub partitions: Partitions,
  pub levels: Vec<Level>,
}

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
  pub levels: HashMap<String, u32>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Partitions {
  pub partitions: HashMap<String, Partition>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Level {
  pub name: String,
  pub object_headers: Vec<ObjectHeader>,
  pub collectables: Vec<Collectable>,
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
#[serde(untagged)]
pub enum ObjectHeader {
  Component(ComponentHeader),
  Actor(ActorHeader),
}

pub enum ObjectHeaderType {
  Circuit,
  Conveyor,
  DroneTransport,
  FreightWagon,
  Game,
  Locomotive,
  PlayerState,
  PowerLine,
  Vehicle,
}

impl ObjectHeader {
  pub fn get_type_path(&self) -> &String {
    match self {
      ObjectHeader::Component(c) => &c.type_path,
      ObjectHeader::Actor(a) => &a.type_path,
    }
  }

  pub fn get_type(&self) -> Option<ObjectHeaderType> {
    let type_path = &self.get_type_path().as_str();

    if CIRCUIT_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::Circuit);
    } else if CONVEYOR_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::Conveyor);
    } else if DRONE_TRANSPORT_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::DroneTransport);
    } else if FREIGHT_WAGON_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::FreightWagon);
    } else if GAME_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::Game);
    } else if LOCOMOTIVE_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::Locomotive);
    } else if PLAYER_STATE_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::PlayerState);
    } else if POWER_LINE_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::PowerLine);
    } else if VEHICLE_PATHS.contains(type_path) {
      return Some(ObjectHeaderType::Vehicle);
    }

    None
  }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentHeader {
  pub type_path: String,
  pub root_object: Option<String>,
  pub instance_name: String,
  pub parent_actor_name: String,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Quaternion<T> {
  pub x: T,
  pub y: T,
  pub z: T,
  pub w: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Vector2D<T> {
  pub x: T,
  pub y: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Vector<T> {
  pub x: T,
  pub y: T,
  pub z: T,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Vector4<T> {
  pub a: T,
  pub b: T,
  pub c: T,
  pub d: T,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ActorHeader {
  pub type_path: String,
  pub root_object: Option<String>,
  pub instance_name: String,
  pub needs_transform: i32,
  pub rotation: Quaternion<f32>,
  pub position: Vector<f32>,
  pub scale: Vector<f32>,
  pub was_placed_in_level: i32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ComponentObject {
  pub should_be_nulled: bool, // Virtual property
  pub save_version: i32,
  pub size_bytes: i32,
  pub properties: Vec<Property>,
  pub missing: Option<String>,
  pub extra: Option<ObjectExtra>,
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
  pub missing: Option<String>,
  pub extra: Option<ObjectExtra>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DroneTransportAction {
  pub name: String,
  pub properties: Vec<Property>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct DroneTransport {
  pub unk_int_1: i32,
  pub unk_int_2: i32,
  pub active_action: Vec<DroneTransportAction>,
  pub action_queue: Vec<DroneTransportAction>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PlayerState {
  pub count: i32,
  pub eos_id: Option<String>,
  pub steam_id: Option<String>,
  pub platform_id: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Circuit {
  pub id: i32,
  pub level_name: String,
  pub path_name: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Conveyor {
  pub length: i32,
  pub name: String,
  pub position: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Locomotive {
  pub name: String,
  pub unk_str_1: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LocomotiveExtra {
  pub count: i32,
  pub elements: Vec<Locomotive>,
  pub prev: ObjectReference,
  pub next: ObjectReference,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PowerLine {
  pub count: i32,
  pub source: ObjectReference,
  pub target: ObjectReference,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Vehicle {
  pub name: String,
  pub unk_str_1: String,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Extra<T> {
  pub count: i32,
  pub elements: Vec<T>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ObjectExtra {
  Circuit(Extra<Circuit>),
  Conveyor(Extra<Conveyor>),
  DroneTransport(DroneTransport),
  Game(Extra<ObjectReference>),
  Locomotive(LocomotiveExtra),
  PlayerState(PlayerState),
  PowerLine(PowerLine),
  Vehicle(Extra<Vehicle>),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
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

  pub fn set_extra(&mut self, extra: ObjectExtra) {
    match self {
      Object::Actor(a) => a.extra = Some(extra),
      Object::Component(c) => c.extra = Some(extra),
    }
  }

  pub fn set_missing(&mut self, missing: String) {
    match self {
      Object::Actor(a) => a.missing = Some(missing),
      Object::Component(c) => c.missing = Some(missing),
    }
  }
}

// This is the same as a Collectable but
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
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
