pub mod callback;
pub mod entity;
pub mod error;
pub mod evaluate;
pub mod primitive;
mod utils;

use self::evaluate::Evaluate;
use self::primitive::Primitive;

use super::parser::parser::*;
use anyhow::Result;
use callback::Actions;
use callback::CallbackResult;
use cgmath::Angle;
use cgmath::Vector2;
use entity::*;
use std::collections::HashMap;
use thiserror::Error;
use utils::{remove_sorted_indices, swap_remove_all};

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
        if let Some(new_actions) = entity.compile_behavior(
            &self.paths,
            &self.patterns,
            &self.prefabs,
            &self.bullets,
            self.fps,
        ) {
            self.actions.push(Some(new_actions));
        } else {
            println!("entity made no further actions -- empty in array!");
            self.actions.push(None);
        }
    }

    pub fn spawn_named(&mut self, name: String) {
        self.entities
            .push(ExecutionEnvironment::new(&self.prefabs[&name]));
    }

    pub fn move_entities(exec: &mut Vec<ExecutionEnvironment>, paths: &PathMap, fps: u16) {
        let get_primitive = |var: String, path: &PathData, vals: &Values| {
            path.definitions
                .get(&var)
                .unwrap()
                .clone()
                .eval(&vals)
                .unwrap()
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
            let mut vals: Values = HashMap::new();
            vals.insert(
                "t".to_string(),
                ExpressionType::Int(environment.elapsed as i64),
            );
            if let Some(pos_fn) = &environment.entity.position_fn {
                if let Some(path) = paths.get(pos_fn) {
                    let x = extract_numeric(get_primitive("x".to_string(), path, &vals));
                    let y = extract_numeric(get_primitive("y".to_string(), path, &vals));
                    environment.entity.position = Vector2::new(x, y);
                }
            } else {
                if let Some(speed) = &environment.entity.speed {
                    let x = *speed * environment.entity.rotation.cos() as f64;
                    let y = *speed * environment.entity.rotation.sin() as f64;
                    environment.entity.velocity = Vector2::new(x, y);
                }
                if let Some(vel_fn) = &environment.entity.velocity_fn {
                    if let Some(path) = paths.get(vel_fn) {
                        let x = extract_numeric(get_primitive("x".to_string(), path, &vals));
                        let y = extract_numeric(get_primitive("y".to_string(), path, &vals));
                        environment.entity.velocity = Vector2::new(x, y);
                    }
                }

                environment.entity.position += environment.entity.velocity * (1.0 / fps as f64);
            }
            environment.elapsed += 1;
        }
    }

    pub fn step(&mut self) {
        // collect all new emplacements per frame
        println!("WORLD STEP\n---------");

        let mut pooled_new_actions: Actions = Vec::new();
        let mut pooled_new_entities: Vec<ExecutionEnvironment> = Vec::new();
        let mut batched_deletions: Vec<usize> = Vec::new();

        // move current entity according to velocity rules

        Interpreter::move_entities(&mut self.entities, &mut self.paths, self.fps);

        // step behavior of each adding new ents to pool: spawns, subpatterns
        println!(
            "entities and actions: {} {}",
            self.entities.len(),
            self.actions.len(),
        );
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
                            println!("running callback for ent {} on frame {}", i, callback.frame);
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
                                    println!("callback returned entities: {}", ents.len());
                                    for ent in &ents {
                                        println!("  adding ent: {:?}\n--", ent);
                                        pooled_new_entities.push(ExecutionEnvironment::new(ent));
                                        let new_actions = ent.compile_behavior(
                                            &self.paths,
                                            &self.patterns,
                                            &self.prefabs,
                                            &self.bullets,
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
                    println!("callbacks to remove: {:?}", removed_callback_indices);
                    swap_remove_all(actions, &removed_callback_indices);
                    self.entities[i].elapsed += 1;
                }
                None => {}
            }
        }
        // sweep the marked dead entities -- a dead entity can have no callbacks
        println!("batched deletions: {:?}", batched_deletions);
        println!("batched additions: {:?}", pooled_new_entities);
        println!("batched additions: {:?}", pooled_new_actions);
        swap_remove_all(&mut self.entities, &batched_deletions);
        swap_remove_all(&mut self.actions, &batched_deletions);
        // add pool to current
        self.entities.append(&mut pooled_new_entities);
        self.actions.append(&mut pooled_new_actions);

        self.elapsed += 1;
    }
}
