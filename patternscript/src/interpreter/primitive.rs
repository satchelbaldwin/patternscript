use super::error::RuntimeError;
use anyhow::Result;

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

pub fn primitive_vec_arithmetic(
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
