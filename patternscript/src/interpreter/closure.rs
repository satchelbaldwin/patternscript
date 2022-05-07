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
    #[error("Vector arithmetic typing error")]
    VecArithTypeError,
    #[error("Vector types can only be int/float/string.")]
    VecTypeError,
    #[error("Type error: Operator {0:?} not defined for types {1:?} and {2:?}")]
    OperatorTypeError(Op, Primitive, Primitive),
}

#[derive(Debug)]
pub enum Primitive {
    I64(i64),
    F64(f64),
    String(String),
    IntVec(Vec<i64>),
    FloatVec(Vec<f64>),
    StrVec(Vec<String>),
    Bool(bool),
}

#[derive(Debug)]
pub enum PrimitiveVecOp {
    Add,
    Sub,
    Mul,
    Div,
}

fn primitive_vec_arithmetic(
    op: PrimitiveVecOp,
    lhs: Primitive,
    rhs: Primitive,
) -> Result<Primitive> {
    use Primitive::*;
    use PrimitiveVecOp::*;
    fn zipmap_vec<A, B, C>(func: fn((&A, &B)) -> C, lhs: &[A], rhs: &[B]) -> Vec<C> {
        lhs.iter().zip(rhs.iter()).map(func).collect()
    }
    match (lhs, rhs) {
        // longhand as some i (X) i -> f, such as 2: i / 4: i -> f
        (IntVec(lh), IntVec(rh)) => match op {
            Add => Ok(IntVec(zipmap_vec(|(l, r)| l + r, &lh, &rh))),
            Sub => Ok(IntVec(zipmap_vec(|(l, r)| l - r, &lh, &rh))),
            Mul => Ok(FloatVec(zipmap_vec(
                |(l, r)| *l as f64 * *r as f64,
                &lh,
                &rh,
            ))),
            Div => Ok(FloatVec(zipmap_vec(
                |(l, r)| *l as f64 / *r as f64,
                &lh,
                &rh,
            ))),
        },
        // shorthand syntax as all f (X) f -> f
        (FloatVec(lh), FloatVec(rh)) => Ok(FloatVec(zipmap_vec(
            match op {
                Add => |(l, r)| l + r,
                Sub => |(l, r)| l - r,
                Mul => |(l, r)| l * r,
                Div => |(l, r)| l / r,
            },
            &lh,
            &rh,
        ))),
        // shorthand, all f (X) i -> f and i (X) f -> f
        // note: we need to matches here to preserve ordering as x: intvec - y: floatvec != x: floatvec - y: floatvec
        //       sub/div are not commutative, and binding types as same name will remove the information on ordering
        (IntVec(lh), FloatVec(rh)) => Ok(FloatVec(zipmap_vec(
            match op {
                // annotating one fixes coercion
                Add => |(l, r): (&i64, &f64)| *l as f64 + r,
                Sub => |(l, r)| *l as f64 - r,
                Mul => |(l, r)| *l as f64 * r,
                Div => |(l, r)| *l as f64 / r,
            },
            &lh,
            &rh,
        ))),

        (FloatVec(lh), IntVec(rh)) => Ok(FloatVec(zipmap_vec(
            match op {
                Add => |(l, r): (&f64, &i64)| l + *r as f64,
                Sub => |(l, r)| l - *r as f64,
                Mul => |(l, r)| l * *r as f64,
                Div => |(l, r)| l / *r as f64,
            },
            &lh,
            &rh,
        ))),
        _ => (Err(RuntimeError::VecArithTypeError.into())),
    }
}

pub trait Evaluable {
    fn eval(self, v: &Values) -> Result<Primitive>;
}
impl Evaluable for ArithmeticExpression {
    fn eval(self, v: &Values) -> Result<Primitive> {
        use Primitive::*;
        match self {
            ArithmeticExpression::Unary(op, val) => match op {
                UnaryOperator::FunctionCall(fn_name) => {
                    // todo: defining builtin funcs should be broken into their own file
                    // todo: all functions return 0.0f
                    Ok(F64(0.0))
                }
                UnaryOperator::Negate => match (*val).eval(v)? {
                    F64(f) => Ok(F64(-1.0 * f)),
                    I64(i) => Ok(I64(-1 * i)),
                    _ => Err(RuntimeError::NegateNonInt.into()),
                },
            },
            ArithmeticExpression::Binary(op, lhs, rhs) => match op {
                // for repeated inner functions, they have to be repeated so that the inner typing is different
                // match arms need the same types
                // todo: refactor for macros at some point?
                Op::Add => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(I64(l + r)),
                    (F64(l), F64(r)) => Ok(F64(l + r)),
                    (I64(i), F64(f)) | (F64(f), I64(i)) => Ok(F64(i as f64 + f)),
                    (Primitive::String(l), Primitive::String(r)) => Ok(Primitive::String(l + &r)),
                    // for any combination of l, r are either intvec or floatvec
                    (l, r)
                        if (matches!(l, Primitive::IntVec(_) | Primitive::FloatVec(_))
                            && matches!(r, Primitive::IntVec(_) | Primitive::FloatVec(_))) =>
                    {
                        primitive_vec_arithmetic(PrimitiveVecOp::Add, l, r)
                    }
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Sub => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(I64(l - r)),
                    (F64(l), F64(r)) => Ok(F64(l - r)),
                    (I64(i), F64(f)) | (F64(f), I64(i)) => Ok(F64(i as f64 - f)),
                    (l, r)
                        if (matches!(l, Primitive::IntVec(_) | Primitive::FloatVec(_))
                            && matches!(r, Primitive::IntVec(_) | Primitive::FloatVec(_))) =>
                    {
                        primitive_vec_arithmetic(PrimitiveVecOp::Sub, l, r)
                    }
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Mul => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(I64(l * r)),
                    (F64(l), F64(r)) => Ok(F64(l * r)),
                    (I64(i), F64(f)) | (F64(f), I64(i)) => Ok(F64(i as f64 * f)),
                    (l, r)
                        if (matches!(l, Primitive::IntVec(_) | Primitive::FloatVec(_))
                            && matches!(r, Primitive::IntVec(_) | Primitive::FloatVec(_))) =>
                    {
                        primitive_vec_arithmetic(PrimitiveVecOp::Mul, l, r)
                    }
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Div => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(I64(l / r)),
                    (F64(l), F64(r)) => Ok(F64(l / r)),
                    (I64(i), F64(f)) | (F64(f), I64(i)) => Ok(F64(i as f64 / f)),
                    (l, r)
                        if (matches!(l, Primitive::IntVec(_) | Primitive::FloatVec(_))
                            && matches!(r, Primitive::IntVec(_) | Primitive::FloatVec(_))) =>
                    {
                        primitive_vec_arithmetic(PrimitiveVecOp::Div, l, r)
                    }
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Exp => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(I64(l.pow(r.try_into().unwrap()))),
                    (F64(l), F64(r)) => Ok(F64(l.powf(r))),
                    (I64(l), F64(r)) => Ok(F64((l as f64).powf(r))),
                    (F64(l), I64(r)) => Ok(F64(l.powf(r as f64))),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::And => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (Bool(l), Bool(r)) => Ok(Bool(l && r)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Or => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (Bool(l), Bool(r)) => Ok(Bool(l || r)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::Test => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (Bool(l), Bool(r)) => Ok(Bool(l == r)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::GT => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(Bool(l > r)),
                    (F64(l), F64(r)) => Ok(Bool(l > r)),
                    (I64(l), F64(r)) => Ok(Bool(l as f64 > r)),
                    (F64(l), I64(r)) => Ok(Bool(l > r as f64)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::GTE => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(Bool(l >= r)),
                    (F64(l), F64(r)) => Ok(Bool(l >= r)),
                    (I64(l), F64(r)) => Ok(Bool(l as f64 >= r)),
                    (F64(l), I64(r)) => Ok(Bool(l >= r as f64)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::LT => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(Bool(l < r)),
                    (F64(l), F64(r)) => Ok(Bool(l < r)),
                    (I64(l), F64(r)) => Ok(Bool((l as f64) < r)),
                    (F64(l), I64(r)) => Ok(Bool(l < r as f64)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
                Op::LTE => match (lhs.eval(v)?, rhs.eval(v)?) {
                    (I64(l), I64(r)) => Ok(Bool(l <= r)),
                    (F64(l), F64(r)) => Ok(Bool(l <= r)),
                    (I64(l), F64(r)) => Ok(Bool((l as f64) <= r)),
                    (F64(l), I64(r)) => Ok(Bool(l <= r as f64)),
                    (l, r) => Err(RuntimeError::OperatorTypeError(op, l, r).into()),
                },
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
                .clone()
                .eval(v),
            ExpressionType::Expr(e) => e.eval(v),
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
                    _ => Err(RuntimeError::VecTypeError.into()),
                }
            }
            ExpressionType::Block(_)
            | ExpressionType::Duration(_)
            | ExpressionType::Range(..)
            | ExpressionType::None => Err(RuntimeError::ComputeTypeError.into()),
        }
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
            Node::Bullet(bd) => {}
            Node::For(fd) => {}
            Node::Head(hd) => {}
            Node::Path(pd) => {}
            Node::Pattern(pd) => {
                let mut inner_values = values.clone();
                inner_values.extend(pd.block.definitions);
                for statement in pd.block.statements {
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
            Node::Spawn(sd) => {}
            // TODO WAIT //
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
        let mut time: u32 = 0;
        self.create_inner(&mut time, HashMap::new(), paths, patterns, ents, fps)
    }
}
