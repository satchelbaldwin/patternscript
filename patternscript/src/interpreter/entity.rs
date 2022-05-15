use std::collections::HashMap;

use super::callback::{Callback, TimedCallback};
use super::evaluate::Evaluate;
use super::primitive::Primitive;
use super::{BulletMap, EntityMap, PathMap, PatternMap};
use crate::parser::parser::{ArithmeticExpression, ExpressionType, Node, UnaryOperator, Values};
use cgmath::{Angle, Deg, Vector2, Vector3};

#[derive(Clone, Debug)]
pub enum VelocityType {
    Simple(Vector2<f64>),
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
    // path funcs are represented as values, args are listed in the values,
    // x and y are given as things that can be evaled by their own scope
    pub position_fn: Option<Values>,
    pub velocity_fn: Option<Values>,

    pub color: Vector3<u8>,
    pub hitbox: Hitbox,
    pub behavior: Behavior,

    pub instance_vars: Option<Values>,
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
            instance_vars: None,
        }
    }

    pub fn compile_behavior(
        &self,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        bullets: &BulletMap,
        globals: Values,
        fps: u16,
    ) -> Option<Vec<TimedCallback<'a>>> {
        match &self.behavior {
            Behavior::Pattern(pd) => Some(
                Node::Pattern(patterns.get(pd)?.clone())
                    .create(paths, patterns, ents, bullets, &globals, fps),
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

    pub fn align_function_args(arg_list: &ExpressionType, arg_vals: &ExpressionType) -> Values {
        // precondition:
        //   arg_list and arg_vals are
        //   arg_list.len() == arg_vals.len()
        // create values from list of arguments and their respective values
        // note:
        //   for paths this will make an infinitely recursive eval on the definitions: t -> t, this is
        //   replaced by the time at runtime
        let mut vals: Values = HashMap::new();
        if let ExpressionType::Vector(arg_list) = arg_list {
            if let ExpressionType::Vector(arg_vals) = arg_vals {
                for i in 0..arg_list.len() {
                    if let ExpressionType::Variable(lhs) = &arg_list[i] {
                        vals.insert(lhs.clone(), arg_vals[i].clone());
                    }
                }
            }
        }
        vals
    }

    /// Constructs a new `Entity`, overriding the defaults.
    ///
    /// # Examples
    ///
    /// ```pattern
    /// bullet mid_sized = {
    ///      sprite = "gameasset";
    ///      hitbox = (4, 4);
    ///      shape = "rectangle";
    ///      color = (255, 255, 0);
    ///      lifetime = 400;
    /// }
    ///
    /// ...
    ///
    /// (within a block)
    /// spawn {
    ///     type = mid_sized;
    ///     position = origin;
    ///     rotation = angle;
    ///     speed = 200;
    ///     lifetime = 800;
    /// }
    /// ```
    ///
    /// will construct an `Entity` first from the default base, then replace the values
    /// with that of the mid_sized bullet definition. After that, the newest definitions
    /// within the spawning block will be used -- that is, the end lifetime will be 800.
    pub fn from_values(
        values: &Values,
        paths: &PathMap,
        bullets: &BulletMap,
        globals: Values,
        instance_vals: Option<Values>,
    ) -> Self {
        let mut entity = Entity::new();

        let mut values = values.clone();
        let original_vals = values.clone();
        entity.instance_vars = instance_vals.clone();
        if let Some(ref mut iv) = entity.instance_vars {
            iv.extend(globals.clone());
            values.extend(iv.clone());
        }

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

        // spawn data

        if let Some(lifetime) = values.get("lifetime") {
            entity.lifetime = match lifetime.clone().eval(&values) {
                Ok(Primitive::I64(i)) => i as u32,
                Ok(Primitive::F64(f)) => f as u32,
                _ => 600,
            }
        }
        if let Some(ExpressionType::Expr(ArithmeticExpression::Unary(
            UnaryOperator::FunctionCall(path_fn_name),
            arguments,
        ))) = values.get("position_fn")
        {
            if let Some(path) = paths.get(path_fn_name) {
                let mut path_vals = Entity::align_function_args(&path.arguments.clone(), arguments);
                path_vals.extend(path.definitions.clone());
                entity.position_fn = Some(path_vals);
            }
        }
        if let Some(ExpressionType::Expr(ArithmeticExpression::Unary(
            UnaryOperator::FunctionCall(path_fn_name),
            arguments,
        ))) = values.get("velocity_fn")
        {
            if let Some(path) = paths.get(path_fn_name) {
                let mut path_vals = Entity::align_function_args(&path.arguments.clone(), arguments);
                path_vals.extend(path.definitions.clone());
                entity.velocity_fn = Some(path_vals);
            }
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
        if let Some(rotation) = values.get("rotation") {
            entity.rotation = match rotation.clone().eval(&values) {
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
        entity
    }
}
