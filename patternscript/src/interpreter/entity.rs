use crate::interpreter::{EntityMap, PathMap, PatternMap};
use crate::parser::parser::PatternData;
use cgmath::{Deg, Vector2, Vector3};
use std::fmt;

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

pub type Actions<'a> = Vec<Vec<TimedCallback<'a>>>;

#[derive(Debug)]
pub enum CallbackResult {
    Delete,
    Mutate,
    AddEntities(Vec<Entity>),
}
// will fire on execution frame >= frame
pub struct EntityCallback<'a>(
    pub  Box<
        dyn 'a + Fn(&mut ExecutionEnvironment, &PathMap, &PatternMap, &EntityMap) -> CallbackResult,
    >,
);

#[derive(Debug)]
pub struct TimedCallback<'a> {
    pub func: EntityCallback<'a>,
    pub frame: u16,
}

impl<'a> fmt::Debug for EntityCallback<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function>")
    }
}

impl<'a> TimedCallback<'a> {
    pub fn new(
        c: impl 'a + Fn(&mut ExecutionEnvironment, &PathMap, &PatternMap, &EntityMap) -> CallbackResult,
        frame: u16,
    ) -> Self {
        TimedCallback {
            func: EntityCallback(Box::new(c)),
            frame,
        }
    }
}

#[derive(Debug)]
pub struct ExecutionEnvironment {
    pub elapsed: u16,
    pub duration: u16,
    pub current_wait: u16,
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
    pub fn compile_behavior(&self) -> Vec<TimedCallback<'a>> {
        return Vec::new();
    }
}
