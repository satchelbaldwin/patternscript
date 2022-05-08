use super::error::*;
use super::primitive::*;
use super::*;
use crate::parser::types::Op;
use anyhow::{Context, Result};

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

impl Evaluable for Condition {
    fn eval(self, v: &Values) -> Result<Primitive> {
        use Primitive::Bool;
        match self {
            Condition::None => Ok(Bool(true)),
            Condition::When(e) => {
                if let Bool(b) = e.eval(v)? {
                    Ok(Bool(b))
                } else {
                    Err(RuntimeError::CondNotBoolError.into())
                }
            }
            Condition::Unless(e) => {
                if let Bool(b) = e.eval(v)? {
                    Ok(Bool(!b))
                } else {
                    Err(RuntimeError::CondNotBoolError.into())
                }
            }
        }
    }
}
