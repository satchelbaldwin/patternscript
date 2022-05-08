use super::callback::{Callback, CallbackResult, EntityCallback, TimedCallback};
use super::{EntityMap, PathMap, PatternMap};
use crate::parser::parser::{ExpressionType, Node, Values};
use cgmath::{Deg, Vector2, Vector3};

pub type PathFn = fn(u64, Vec<f64>) -> Vector2<f64>;

#[derive(Clone, Debug)]
pub enum VelocityType {
    Simple,
    Acceleration(Vector2<f64>),
    Path(PathFn),
}

#[derive(Clone, Debug)]
pub enum HitboxType {
    Rectangle,
    Ellipse,
}
#[derive(Clone, Debug)]
pub struct Hitbox {
    pub size: Vector2<u16>,
    pub offset: Vector2<f64>,
    pub hitbox_type: HitboxType,
}

#[derive(Clone, Debug)]
pub enum Behavior {
    Pattern(String),
    Simple,
}
#[derive(Clone, Debug)]
pub enum EntityType {}

#[derive(Clone, Debug)]
pub struct Entity {
    pub position: Vector2<f64>,
    pub velocity: Vector2<f64>,
    pub rotation: Deg<u16>,
    pub lifetime: u64,

    pub color: Vector3<u8>,
    pub hitbox: Hitbox,
    pub behavior: Behavior,
}

#[derive(Debug)]
pub struct ExecutionEnvironment {
    pub elapsed: u32,
    pub duration: u32,
    pub current_wait: u32,
    pub entity: Entity,
}

impl ExecutionEnvironment {
    pub fn new(e: &Entity) -> Self {
        ExecutionEnvironment {
            elapsed: 0,
            duration: 0,
            current_wait: 0,
            entity: e.clone(),
        }
    }
}

impl<'a> Entity {
    pub fn new() -> Self {
        Entity {
            position: Vector2 { x: 0.0, y: 0.0 },
            velocity: Vector2 { x: 0.0, y: 0.0 },
            rotation: Deg(0),
            lifetime: 240,
            color: Vector3 { x: 255, y: 0, z: 0 },
            hitbox: Hitbox {
                size: Vector2 { x: 3, y: 3 },
                offset: Vector2 { x: 0.0, y: 0.0 },
                hitbox_type: HitboxType::Rectangle,
            },
            behavior: Behavior::Simple,
        }
    }

    pub fn compile_behavior(
        &self,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        fps: u16,
    ) -> Option<Vec<TimedCallback<'a>>> {
        println!("behavior: ent");
        match &self.behavior {
            Behavior::Pattern(pd) => {
                Some(Node::Pattern(patterns.get(pd)?.clone()).create(paths, patterns, ents, fps))
            }
            Behavior::Simple => None,
        }
    }

    pub fn from_values(values: &Values) -> Self {
        let mut entity = Entity::new();
        if let ExpressionType::String(e_type) = values.get("type").unwrap() {
            // this should be fixed
        }
        entity
    }
}
