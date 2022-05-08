pub mod callback;
pub mod entity;
pub mod error;
pub mod evaluate;
pub mod primitive;
mod utils;

use super::parser::parser::*;
use anyhow::Result;
use callback::Actions;
use callback::CallbackResult;
use entity::*;
use std::collections::HashMap;
use thiserror::Error;
use utils::remove_sorted_indices;

// todo: move IError to RuntimeError after refactor
#[derive(Debug, Error)]
pub enum IError {
    #[error("Parse of pattern did not result in head node.")]
    FromParse,
}

type PathMap = HashMap<String, PathFn>;
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
        // todo: finish committing to map
        println!("registering path: {}", name);
        let args = &pd.arguments;
        let fnc = |t: u64, args: Vec<f64>| {};
        println!("{:?}", args);
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
        if let Some(new_actions) =
            entity.compile_behavior(&self.paths, &self.patterns, &self.prefabs, self.fps)
        {
            self.actions.push(new_actions);
        }
    }

    pub fn spawn_named(&mut self, name: String) {
        self.entities
            .push(ExecutionEnvironment::new(&self.prefabs[&name]));
    }

    pub fn step(&mut self) {
        // collect all new emplacements per frame
        let mut pooled_new_actions: Actions = Vec::new();
        let mut pooled_new_entities: Vec<ExecutionEnvironment> = Vec::new();
        let mut batched_deletions: Vec<usize> = Vec::new();

        // move current entity according to velocity rules
        for i in 0..self.entities.len() {}

        // step behavior of each adding new ents to pool: spawns, subpatterns
        for i in 0..self.entities.len() {
            let mut removed_callback_indices: Vec<usize> = Vec::new();
            for callback_index in 0..self.actions[i].len() {
                let callback = &self.actions[i][callback_index];
                if callback.frame >= self.entities[i].elapsed {
                    removed_callback_indices.push(callback_index);
                    let result = (*callback.func.0)(
                        &mut self.entities[i],
                        &self.paths,
                        &self.patterns,
                        &self.prefabs,
                    );
                    match result {
                        CallbackResult::AddEntities(ents) => {
                            for ent in &ents {
                                pooled_new_entities.push(ExecutionEnvironment::new(ent));
                                if let Some(new_actions) = ent.compile_behavior(
                                    &self.paths,
                                    &self.patterns,
                                    &self.prefabs,
                                    self.fps,
                                ) {
                                    pooled_new_actions.push(new_actions);
                                }
                            }
                        }
                        CallbackResult::Delete => batched_deletions.push(i),
                        CallbackResult::Mutate => {}
                    }
                }
            }

            self.entities[i].elapsed += 1;
        }
        // sweep the marked dead entities
        remove_sorted_indices(&mut self.entities, batched_deletions.clone());
        remove_sorted_indices(&mut self.actions, batched_deletions);
        // add pool to current
        self.entities.append(&mut pooled_new_entities);
        self.actions.append(&mut pooled_new_actions);

        self.elapsed += 1;
    }
}
