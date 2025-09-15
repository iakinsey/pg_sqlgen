use candle_core::Tensor;

use crate::types::errors::UtilError;

pub fn l2_norm(v: &Tensor) -> Result<Tensor, UtilError> {
    Ok(v.broadcast_div(&v.sqr()?.sum_keepdim(1)?.sqrt()?)?)
}
