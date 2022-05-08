use super::primitive::Primitive;
use crate::parser::types::Op;
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
    #[error("Conditional didn't evaluate to boolean type")]
    CondNotBoolError,
}
