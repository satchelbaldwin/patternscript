use super::error::*;
use super::evaluate::Evaluate;
use super::primitive::*;
use crate::parser::parser::{ExpressionType, Values};
use anyhow::Result;
use cgmath::{Angle, Deg};

pub fn dispatch_func(
    fn_name: String,
    values: &Values,
    arg: Box<ExpressionType>,
) -> Result<Primitive> {
    match fn_name.as_str() {
        "sqrt" => sqrt(values, arg),
        "sin" => sin(values, arg),
        "cos" => cos(values, arg),
        "tan" => tan(values, arg),
        "x" => access_x(values, arg),
        "y" => access_y(values, arg),
        _ => (unimplemented!()),
    }
}

// todo: make this better and add array indexing to language

fn sqrt(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    use Primitive::*;
    match arg.eval(values)? {
        I64(i) => Ok(F64((i as f64).sqrt())),
        F64(f) => Ok(F64(f.sqrt())),
        _ => Err(RuntimeError::Generic.into()),
    }
}

fn sin(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    use Primitive::*;
    match arg.eval(values)? {
        I64(i) => Ok(F64(Deg(i as f64).sin())),
        F64(f) => Ok(F64(Deg(f).sin())),
        _ => Err(RuntimeError::Generic.into()),
    }
}

fn cos(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    use Primitive::*;
    match arg.eval(values)? {
        I64(i) => Ok(F64(Deg(i as f64).cos())),
        F64(f) => Ok(F64(Deg(f).cos())),
        _ => Err(RuntimeError::Generic.into()),
    }
}

fn tan(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    use Primitive::*;
    match arg.eval(values)? {
        I64(i) => Ok(F64(Deg(i as f64).tan())),
        F64(f) => Ok(F64(Deg(f).tan())),
        _ => Err(RuntimeError::Generic.into()),
    }
}

fn access_x(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    match arg.eval(values)? {
        Primitive::IntVec(i) => Ok(Primitive::I64(i[0])),
        Primitive::FloatVec(f) => Ok(Primitive::F64(f[0])),
        Primitive::StrVec(s) => Ok(Primitive::String(s[0].clone())),
        _ => Err(RuntimeError::Generic.into()),
    }
}

fn access_y(values: &Values, arg: Box<ExpressionType>) -> Result<Primitive> {
    match arg.eval(values)? {
        Primitive::IntVec(i) => Ok(Primitive::I64(i[1])),
        Primitive::FloatVec(f) => Ok(Primitive::F64(f[1])),
        Primitive::StrVec(s) => Ok(Primitive::String(s[1].clone())),
        _ => Err(RuntimeError::Generic.into()),
    }
}
