mod de;
mod error;
mod trace;

pub use error::Error;

use serde::de::{Deserialize, DeserializeSeed};
use trace::{trace, trace_seed_copy};

pub fn from_args<'de, D>() -> Result<D, Error>
where
    D: Deserialize<'de>,
{
    let shape = trace::<D>()?;

    todo!()
}

pub fn from_args_seed<'de, D>(seed: D) -> Result<D::Value, Error>
where
    D: Copy + DeserializeSeed<'de>,
{
    let shape = trace_seed_copy(seed)?;

    todo!()
}
