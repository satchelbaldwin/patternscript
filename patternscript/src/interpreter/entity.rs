use super::callback::{Callback, CallbackResult, EntityCallback, TimedCallback};
use super::evaluate::Evaluate;
use super::primitive::Primitive;
use super::{BulletMap, EntityMap, PathMap, PatternMap};
use crate::parser::parser::{ExpressionType, Node, Values};
use cgmath::{Angle, Deg, Vector2, Vector3};

#[derive(Clone, Debug)]
pub enum VelocityType {
    Simple,
    Acceleration(Vector2<f64>),
    Path(ExpressionType),
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
    pub rotation: Deg<f32>,
    pub speed: Option<f64>,
    pub lifetime: u32,
    pub position_fn: Option<String>,
    pub velocity_fn: Option<String>,

    pub color: Vector3<u8>,
    pub hitbox: Hitbox,
    pub behavior: Behavior,
}

#[derive(Debug, Clone)]
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
            duration: e.lifetime,
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
            rotation: Deg(0.),
            speed: None,
            lifetime: 600,
            color: Vector3 { x: 255, y: 0, z: 0 },
            hitbox: Hitbox {
                size: Vector2 { x: 3, y: 3 },
                offset: Vector2 { x: 0.0, y: 0.0 },
                hitbox_type: HitboxType::Rectangle,
            },
            behavior: Behavior::Simple,
            position_fn: None,
            velocity_fn: None,
        }
    }

    pub fn compile_behavior(
        &self,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        bullets: &BulletMap,
        fps: u16,
    ) -> Option<Vec<TimedCallback<'a>>> {
        match &self.behavior {
            Behavior::Pattern(pd) => Some(
                Node::Pattern(patterns.get(pd)?.clone())
                    .create(paths, patterns, ents, bullets, fps),
            ),
            Behavior::Simple => None,
        }
    }

    pub fn extract_color(expression: &ExpressionType, values: &Values) -> Vector3<u8> {
        if let ExpressionType::Vector(ve) = expression {
            let color: Vec<u8> = ve
                .iter()
                .map(|e| {
                    if let Ok(p) = e.clone().eval(&values) {
                        match p {
                            Primitive::I64(i) => i as u8,
                            Primitive::F64(f) => (f * 255.) as u8,
                            _ => 0,
                        }
                    } else {
                        0
                    }
                })
                .collect();
            return Vector3::new(color[0], color[1], color[2]);
        } else {
            Vector3::new(255, 0, 0)
        }
    }

    pub fn from_values(values: &Values, bullets: &BulletMap) -> Self {
        let mut entity = Entity::new();

        let mut values = values.clone();
        let original_vals = values.clone();

        // bullet prefab data
        if let Some(ExpressionType::Variable(e_type)) = values.get("type") {
            if let Some(prefab) = bullets.get(e_type) {
                for (k, v) in &prefab.definitions {
                    // TODO: SPRITE, HITBOX, SHAPE
                    match k.as_str() {
                        "color" => {
                            entity.color = Entity::extract_color(v, &values);
                        }
                        _ => {}
                    }
                }
                values.extend(prefab.definitions.clone());
                // re-extend originals for overrides
                values.extend(original_vals);
            }
        }

        println!("{:?}", values);

        // spawn data

        if let Some(lifetime) = values.get("lifetime") {
            println!("Lifetime");
            entity.lifetime = match lifetime.clone().eval(&values) {
                Ok(Primitive::I64(i)) => i as u32,
                Ok(Primitive::F64(f)) => f as u32,
                _ => 600,
            }
        }
        if let Some(ExpressionType::Variable(path_fn)) = values.get("position_fn") {
            entity.position_fn = Some(path_fn.clone());
        }
        if let Some(ExpressionType::Variable(path_fn)) = values.get("velocity_fn") {
            entity.velocity_fn = Some(path_fn.clone());
        }
        if let Some(position) = values.get("position") {
            entity.position = match position.clone().eval(&values) {
                Ok(Primitive::FloatVec(f)) => Vector2::new(f[0], f[1]),
                Ok(Primitive::IntVec(i)) => Vector2::new(i[0] as f64, i[1] as f64),
                _ => Vector2::new(0.0, 0.0),
            }
        }
        if let Some(velocity) = values.get("velocity") {
            entity.velocity = match velocity.clone().eval(&values) {
                Ok(Primitive::FloatVec(f)) => Vector2::new(f[0], f[1]),
                Ok(Primitive::IntVec(i)) => Vector2::new(i[0] as f64, i[1] as f64),
                _ => Vector2::new(0.0, 0.0),
            }
        }
        if let Some(direction) = values.get("direction") {
            entity.rotation = match direction.clone().eval(&values) {
                Ok(Primitive::I64(i)) => Deg(i as f32).normalize(),
                Ok(Primitive::F64(f)) => Deg(f as f32).normalize(),
                _ => Deg(0.0),
            }
        }
        if let Some(speed) = values.get("speed") {
            entity.speed = match speed.clone().eval(&values) {
                Ok(Primitive::I64(i)) => Some(i as f64),
                Ok(Primitive::F64(f)) => Some(f),
                _ => Some(10.0),
            }
        }

        println!("{:?}", entity);

        entity
    }
}
