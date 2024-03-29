pub mod callback;
pub mod entity;
pub mod error;
pub mod evaluate;
pub mod primitive;
pub mod ps_funcs;
mod utils;

use self::evaluate::Evaluate;
use self::primitive::Primitive;

use super::parser::parser::*;
use anyhow::Result;
use callback::{Actions, CallbackResult};
use cgmath::{Angle, Vector2};
use entity::*;
use std::collections::HashMap;
use thiserror::Error;
use utils::swap_remove_all;

// todo: move IError to RuntimeError after refactor
#[derive(Debug, Error)]
pub enum IError {
    #[error("Parse of pattern did not result in head node.")]
    FromParse,
}

type PathMap = HashMap<String, PathData>;
type EntityMap = HashMap<String, Entity>;
type PatternMap = HashMap<String, PatternData>;
type BulletMap = HashMap<String, BulletData>;

#[derive(Debug)]
pub struct Interpreter<'a> {
    pub elapsed: u64,
    pub fps: u16,
    pub head: HeadData,
    pub entities: Vec<ExecutionEnvironment>,
    pub actions: Actions<'a>,
    pub paths: PathMap,
    pub prefabs: EntityMap,
    pub patterns: PatternMap,
    pub bullets: BulletMap,
}

impl<'a> Interpreter<'a> {
    pub fn new(hd: HeadData) -> Self {
        let mut i = Interpreter {
            elapsed: 0,
            fps: 120,
            head: hd,
            entities: Vec::new(),
            actions: Vec::new(),
            paths: HashMap::new(),
            prefabs: HashMap::new(),
            patterns: HashMap::new(),
            bullets: HashMap::new(),
        };
        i.initialize();
        i
    }

    pub fn from_parse_result(n: Node) -> Result<Self> {
        match n {
            Node::Head(hd) => Ok(Interpreter::new(hd)),
            _ => Err(IError::FromParse.into()),
        }
    }

    pub fn initialize(&mut self) {
        for (k, v) in &self.head.definitions {
            match v {
                Node::Path(pd) => Interpreter::register_path(k, &mut self.paths, pd),
                Node::Pattern(pd) => Interpreter::register_pattern(k, &mut self.patterns, pd),
                Node::Bullet(bd) => Interpreter::register_bullet(k, &mut self.bullets, bd),
                _ => {}
            }
        }
    }

    fn register_path(name: &String, paths: &mut PathMap, pd: &PathData) {
        println!("registering path: {}", name);
        paths.insert(name.clone(), pd.clone());
    }

    fn register_pattern(name: &String, patterns: &mut PatternMap, pd: &PatternData) {
        // todo: finish committing to map
        println!("registering pattern: {}", name);
        patterns.insert(name.clone(), pd.clone());
    }

    fn register_bullet(name: &String, bullets: &mut BulletMap, bd: &BulletData) {
        // todo: finish committing to map
        println!("registering bullet: {}", name);
        bullets.insert(name.clone(), bd.clone());
    }

    pub fn spawn_direct(&mut self, entity: &Entity) {
        self.entities.push(ExecutionEnvironment::new(entity));
        let globals = Interpreter::create_globals(entity.position);
        if let Some(new_actions) = entity.compile_behavior(
            &self.paths,
            &self.patterns,
            &self.prefabs,
            &self.bullets,
            globals,
            self.fps,
        ) {
            self.actions.push(Some(new_actions));
        } else {
            self.actions.push(None);
        }
    }

    pub fn spawn_named(&mut self, name: String) {
        self.entities
            .push(ExecutionEnvironment::new(&self.prefabs[&name]));
    }

    pub fn move_entities(exec: &mut Vec<ExecutionEnvironment>, fps: u16) {
        let get_primitive = |var: String, vals: &Values| -> Primitive {
            vals.get(&var).unwrap().clone().eval(&vals).unwrap()
        };
        let extract_numeric = |primitive: Primitive| -> f64 {
            return match primitive {
                Primitive::I64(i) => i as f64,
                Primitive::F64(f) => f,
                _ => 0.0,
            };
        };
        for environment in exec {
            // is it rotation/speed or hard set pos/vel?
            // precedence:
            //   position_fn exists
            //   velocity_fn exists (set velocity, resolve position per frame)
            //   speed/rotation exist, resolve velocity, then resolve position from velocity
            //   resolve position from velocity

            // contains patternscript globals to be added in
            //   such as the entity position

            //todo: instance overwrites locals, shouldn't
            let mut vals: Values = environment
                .entity
                .instance_vars
                .clone()
                .unwrap_or(HashMap::new());
            // time
            vals.insert(
                "t".to_string(),
                ExpressionType::Int(environment.elapsed as i64),
            );
            // towards player
            vals.insert(
                "towards_player".to_string(),
                Interpreter::angle_towards_player(),
            );

            if let Some(pos_fn) = &environment.entity.position_fn {
                let mut fn_with_globals = pos_fn.clone();
                fn_with_globals.extend(vals);
                let x = extract_numeric(get_primitive("x".to_string(), &fn_with_globals));
                let y = extract_numeric(get_primitive("y".to_string(), &fn_with_globals));
                environment.entity.position = Vector2::new(x, y);
            } else {
                if let Some(speed) = &environment.entity.speed {
                    let x = *speed * environment.entity.rotation.cos() as f64;
                    let y = *speed * environment.entity.rotation.sin() as f64;
                    environment.entity.velocity = Vector2::new(x, y);
                }
                if let Some(vel_fn) = &environment.entity.velocity_fn {
                    let mut fn_with_globals = vel_fn.clone();
                    fn_with_globals.extend(vals);
                    let x = extract_numeric(get_primitive("x".to_string(), &fn_with_globals));
                    let y = extract_numeric(get_primitive("y".to_string(), &fn_with_globals));
                    environment.entity.velocity = Vector2::new(x, y);
                }

                environment.entity.position += environment.entity.velocity * (1.0 / fps as f64);
            }
            //environment.elapsed += 1;
        }
    }

    fn angle_towards_player() -> ExpressionType {
        // todo: no player
        ExpressionType::Float(0.0)
    }

    fn entity_pos_as_expr(entity_position: Vector2<f64>) -> ExpressionType {
        ExpressionType::Vector(vec![
            ExpressionType::Float(entity_position[0]),
            ExpressionType::Float(entity_position[1]),
        ])
    }

    // create spawn-time globals -- these will not be accurate for per frame movements
    pub fn create_globals(entity_position: Vector2<f64>) -> Values {
        let mut globals: Values = HashMap::new();
        globals.insert(
            "towards_player".to_string(),
            Interpreter::angle_towards_player(),
        );
        globals.insert(
            "entity_position".to_string(),
            Interpreter::entity_pos_as_expr(entity_position),
        );
        globals
    }

    pub fn step(&mut self) {
        // collect all new emplacements per frame
        let mut pooled_new_actions: Actions = Vec::new();
        let mut pooled_new_entities: Vec<ExecutionEnvironment> = Vec::new();
        let mut batched_deletions: Vec<usize> = Vec::new();

        // move current entity according to velocity rules
        Interpreter::move_entities(&mut self.entities, self.fps);

        // step behavior of each adding new ents to pool: spawns, subpatterns
        for i in 0..self.entities.len() {
            let mut removed_callback_indices: Vec<usize> = Vec::new();
            // lifetime outlives, remove and don't check actions
            if self.entities[i].duration <= self.entities[i].elapsed {
                batched_deletions.push(i);
                continue;
            }
            match &mut self.actions[i] {
                Some(actions) => {
                    for callback_index in 0..actions.len() {
                        let callback = &actions[callback_index];
                        if callback.frame <= self.entities[i].elapsed {
                            removed_callback_indices.push(callback_index);
                            let result = (*callback.func.0)(
                                &mut self.entities[i],
                                &self.paths,
                                &self.patterns,
                                &self.prefabs,
                                &self.bullets,
                            );
                            match result {
                                CallbackResult::AddEntities(ents) => {
                                    for ent in &ents {
                                        let globals = Interpreter::create_globals(
                                            self.entities[i].entity.position,
                                        );
                                        pooled_new_entities.push(ExecutionEnvironment::new(ent));
                                        let new_actions = ent.compile_behavior(
                                            &self.paths,
                                            &self.patterns,
                                            &self.prefabs,
                                            &self.bullets,
                                            globals,
                                            self.fps,
                                        );
                                        pooled_new_actions.push(new_actions);
                                    }
                                }
                                CallbackResult::Delete => batched_deletions.push(i),
                                CallbackResult::Mutate => {}
                            }
                        }
                    }
                    // remove singular entity's spent callbacks and advance its lifetime
                    swap_remove_all(actions, &removed_callback_indices);
                    self.entities[i].elapsed += 1;
                }
                None => {}
            }
        }
        // sweep the marked dead entities -- a dead entity can have no callbacks
        swap_remove_all(&mut self.entities, &batched_deletions);
        swap_remove_all(&mut self.actions, &batched_deletions);
        // add pool to current
        self.entities.append(&mut pooled_new_entities);
        self.actions.append(&mut pooled_new_actions);

        self.elapsed += 1;
    }
}
