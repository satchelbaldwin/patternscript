use super::entity::*;
use super::*;
use crate::parser::types::Op;
use anyhow::{Context, Result};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Generic runtime error")]
    Generic,
    #[error("Variable not defined: {0}")]
    VarNotDef(String),
    #[error("Cant compute runtime value of this type")]
    ComputeTypeError,
    #[error("Cannot negate non-integer type")]
    NegateNonInt,
}

trait Callback<'a> {
    fn create(self) -> Vec<TimedCallback<'a>>;
    fn create_inner(self, time: u16, values: Values) -> Vec<TimedCallback<'a>>;
}

enum Primitive {
    I64(i64),
    F64(f64),
    String(String),
    IntVec(Vec<i64>),
    FloatVec(Vec<f64>),
    StrVec(Vec<String>),
}

trait Evaluable {
    fn eval(self, v: &Values) -> Result<Primitive>;
}
impl Evaluable for ArithmeticExpression {
    fn eval(self, v: &Values) -> Result<Primitive> {
        match self {
            ArithmeticExpression::Unary(op, val) => match op {
                UnaryOperator::FunctionCall(fn_name) => {
                    // todo: defining builtin funcs should be broken into their own file
                    // todo: all functions return 0.0f
                    Ok(Primitive::F64(0.0))
                }
                UnaryOperator::Negate => match (*val).eval(v)? {
                    Primitive::F64(f) => Ok(Primitive::F64(-1.0 * f)),
                    Primitive::I64(i) => Ok(Primitive::I64(-1 * i)),
                    _ => Err(RuntimeError::NegateNonInt.into()),
                },
            },
            ArithmeticExpression::Binary(op, lhs, rhs) => match op {
                // todo: finish arithmetic returns
                Op::Test => {}
                _ => {}
            },
        }
    }
}
impl Evaluable for ExpressionType {
    fn eval(self, v: &Values) -> Result<Primitive> {
        match self {
            ExpressionType::String(s) => Ok(Primitive::String(s)),
            ExpressionType::Float(f) => Ok(Primitive::F64(f)),
            ExpressionType::Int(i) => Ok(Primitive::I64(i)),
            ExpressionType::Variable(var) => v
                .get(&var)
                .context(format!("Variable Not Defined: {}", var))?
                .eval(v),
            ExpressionType::Vector(vec) => {
                // empty vectors can't exist in the parser, i think
                match &vec.first().unwrap() {
                    // check first element for type: they must be homogenous
                    // would this be better as a macro?
                    ExpressionType::Int(_i) => {
                        let mut primitive_vec: Vec<i64> = Vec::new();
                        for element in vec {
                            let inner = element.eval(v)?;
                            match inner {
                                Primitive::I64(i) => {
                                    primitive_vec.push(i);
                                }
                                _ => {}
                            }
                        }
                        Ok(Primitive::IntVec(primitive_vec))
                    }
                    ExpressionType::Float(_f) => {
                        let mut primitive_vec: Vec<f64> = Vec::new();
                        for element in vec {
                            let inner = element.eval(v)?;
                            match inner {
                                Primitive::F64(f) => {
                                    primitive_vec.push(f);
                                }
                                _ => {}
                            }
                        }
                        Ok(Primitive::FloatVec(primitive_vec))
                    }
                    ExpressionType::String(_s) => {
                        let mut primitive_vec: Vec<String> = Vec::new();
                        for element in vec {
                            let inner = element.eval(v)?;
                            match inner {
                                Primitive::String(s) => {
                                    primitive_vec.push(s);
                                }
                                _ => {}
                            }
                        }
                        Ok(Primitive::StrVec(primitive_vec))
                    }
                }
            }
            ExpressionType::Expr(e) => e.eval(v),
            ExpressionType::Block(_)
            | ExpressionType::Duration(_)
            | ExpressionType::Range(..)
            | ExpressionType::None => Err(RuntimeError::ComputeTypeError.into()),
        }
    }
}

impl<'a> Callback<'a> for Node {
    fn create_inner(self, time: u16, values: Values) -> Vec<TimedCallback<'a>> {
        let mut result: Vec<TimedCallback<'a>> = Vec::new();

        match self {
            Node::Bullet(bd) => {}
            Node::For(fd) => {}
            Node::Head(hd) => {}
            Node::Path(pd) => {}
            Node::Pattern(pd) => {}
            Node::Spawn(sd) => {}
            Node::Wait(wd) => {}
        }

        result
    }
    fn create(self) -> Vec<TimedCallback<'a>> {
        self.create_inner(0, HashMap::new())
    }
}

impl<'a> Callback<'a> for ExpressionType {
    fn create_inner(self, time: u16, values: Values) -> Vec<TimedCallback<'a>> {
        let mut result: Vec<TimedCallback<'a>> = Vec::new();

        result
    }
    fn create(self) -> Vec<TimedCallback<'a>> {
        self.create_inner(0, HashMap::new())
    }
}
