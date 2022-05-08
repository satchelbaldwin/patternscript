use crate::interpreter::error::RuntimeError;

use super::entity::*;
use super::evaluate::Evaluate;
use super::primitive::*;
use super::*;
use anyhow::Context;
use itertools::Itertools;
use std::fmt;

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
    pub frame: u32,
}

impl<'a> TimedCallback<'a> {
    pub fn new(
        c: impl 'a + Fn(&mut ExecutionEnvironment, &PathMap, &PatternMap, &EntityMap) -> CallbackResult,
        frame: u32,
    ) -> Self {
        TimedCallback {
            func: EntityCallback(Box::new(c)),
            frame,
        }
    }
}

impl<'a> fmt::Debug for EntityCallback<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<function>")
    }
}

pub trait Callback<'a> {
    fn create(
        self,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        fps: u16,
    ) -> Vec<TimedCallback<'a>>;
    fn create_inner(
        self,
        time: &mut u32,
        values: Values,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        fps: u16,
    ) -> Vec<TimedCallback<'a>>;
}

impl<'a> Callback<'a> for Node {
    fn create_inner(
        self,
        time: &mut u32,
        values: Values,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        fps: u16,
    ) -> Vec<TimedCallback<'a>> {
        let mut result: Vec<TimedCallback<'a>> = Vec::new();

        match self {
            Node::Bullet(_bd) => {
                // not sure if these should be inline within pattern rather than top level.
                // todo: revisit grammar for this one
            }
            Node::For(fd) => {
                // create a range from the expression range type
                let range_from_exp = |exp: ExpressionType| -> Option<Vec<i64>> {
                    match exp {
                        ExpressionType::Range(a, b) => {
                            Some((std::ops::Range { start: a, end: b }).collect::<Vec<i64>>())
                        }
                        _ => None,
                    }
                };

                // create new variable bindings for the execution frame

                // O(n1 * n2 * n3) -- they're, in essence, nested for loops
                // patternscript for syntax       -> rust HashMap<String,ExprType}    -> rust Vec<String>, Vec<Vec<i64>> -> rust Vec<HashMap<String,i64>>
                // (i = 0..3, j = 0..3, k = 0..3) -> {x:Expr(0..3), y:Expr(0..3),...} -> [i, j, k] [[0,1,..],[0,..],..]  -> [{i:0, j:0, k:0},{i:1, j:0, k:0},...]
                let mut var_names: Vec<String> = Vec::new();
                let mut var_ranges: Vec<Vec<i64>> = Vec::new();
                for (var, range_expr) in fd.initial_definitions {
                    let range = range_from_exp(range_expr);
                    match range {
                        Some(r) => {
                            var_names.push(var);
                            var_ranges.push(r);
                        }
                        None => (),
                    }
                }

                // contains all combinations of inner for loop variables
                let iterations: Vec<Vec<&i64>> =
                    var_ranges.iter().multi_cartesian_product().collect();
                println!("{:?}", iterations);
                // for each in-patternscript for loop statements body
                for v in iterations {
                    let mut new_bindings: Values = HashMap::new();
                    for i in 0..v.len() {
                        new_bindings.insert(var_names[i].clone(), ExpressionType::Int(*v[i]));
                    }
                    // now perform for loop inner body with execution frame having new bindings
                    // extended over the existing stack frame, allowing nested scope to get the for-binds
                    // plus the existing scope
                    let mut all_bindings = values.clone();
                    all_bindings.extend(new_bindings);

                    // handle according to conditional inclusion/exclusion rules, then execute block
                    if let Ok(Primitive::Bool(b)) = fd.condition.clone().eval(&all_bindings) {
                        if b {
                            // condition passed
                            // todo: optimization: there's a lot of clones of variable bindings, maybe fix this later
                            let owned_block = fd.body.clone();
                            let mut inner_bindings = all_bindings.clone();
                            inner_bindings.extend(owned_block.definitions);
                            for node in owned_block.statements {
                                // finally, parsing the statements themselves
                                let mut rvec = node.create_inner(
                                    time,
                                    inner_bindings.clone(),
                                    paths,
                                    patterns,
                                    ents,
                                    fps,
                                );
                                if rvec.len() > 0 {
                                    result.append(&mut rvec);
                                }
                            }
                        }
                    }
                }
            }
            Node::Head(_hd) => {
                // head should already be parsed into the three reference maps passed
            }
            Node::Path(_pd) => {
                // not sure if these should be inline within pattern rather than top level.
                // todo: revisit grammar for this one
            }
            Node::Pattern(pd) => {
                println!("Pat {:?}", pd);
                let mut inner_values = values.clone();
                inner_values.extend(pd.block.definitions);
                println!("INNER VALS {:?}", inner_values);
                println!("\n\n{:?}\n\n", pd.block.statements);

                if let ExpressionType::String(st) = inner_values
                    .get("iteration_type")
                    .unwrap_or(&ExpressionType::String("blank".to_string()))
                {
                    match st.as_str() {
                        "time" => {
                            //todo: time
                        }
                        "cycles" => {
                            //todo: cycles
                        }
                        _ => {
                            //todo: iter once
                        }
                    }
                }
                for statement in pd.block.statements {
                    println!("\n\npattern statement: {:?} \n\n", statement);
                    let mut r = statement.create_inner(
                        time,
                        inner_values.clone(),
                        paths,
                        patterns,
                        ents,
                        fps,
                    );
                    if r.len() > 0 {
                        result.append(&mut r);
                    }
                }
            }
            Node::Spawn(sd) => {
                // this is the one that creates the callbacks -- that is, messages back to state when ran on x time

                // todo: this is ALL placeholder
                let e = Entity::from_values(&sd.definitions);
                let mut ents: Vec<Entity> = Vec::new();
                ents.push(e);
                let tc = TimedCallback::new(
                    move |ex, path, pat, ent| CallbackResult::AddEntities(ents.clone()),
                    *time,
                );
                result.push(tc)
            }
            Node::Wait(wd) => match wd {
                // parser precondition that waitdata::variants are of specific types
                // frames: int
                // time:   int/float
                WaitData::Frames(f) => {
                    if let ExpressionType::Int(f) = f {
                        *time = *time + f as u32;
                    }
                }
                WaitData::Time(t) => match t {
                    ExpressionType::Int(i) => {
                        // wait negative seconds doesn't make sense//scary cast i64>u32
                        *time = *time + i as u32 * fps as u32;
                    }
                    ExpressionType::Float(f) => *time = *time + (f * fps as f64).floor() as u32,
                    _ => {
                        panic!("this should be caught by the parser, if you see this i made a regression, please report a bug")
                    }
                },
            },
        }

        result
    }
    fn create(
        self,
        paths: &PathMap,
        patterns: &PatternMap,
        ents: &EntityMap,
        fps: u16,
    ) -> Vec<TimedCallback<'a>> {
        println!("create");
        let mut time: u32 = 0;
        self.create_inner(&mut time, HashMap::new(), paths, patterns, ents, fps)
    }
}
