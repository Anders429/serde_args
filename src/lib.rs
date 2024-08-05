mod de;
mod error;
mod parse;
mod trace;

pub use error::Error;

use de::Deserializer;
use parse::parse;
use serde::de::{Deserialize, DeserializeSeed};
use std::{env, marker::PhantomData, path::PathBuf};
use trace::{trace, trace_seed_copy};

pub fn from_args_seed<'de, D>(seed: D) -> Result<D::Value, Error>
where
    D: Copy + DeserializeSeed<'de>,
{
    let mut shape = trace_seed_copy(seed)?;

    dbg!(&shape);

    let mut args = env::args_os();
    let executable_path = PathBuf::from(args.next().ok_or(Error::EmptyArgs)?)
        .file_name()
        .ok_or(Error::MissingExecutableName)?;

    let context = parse(args, &mut shape)?;

    seed.deserialize(Deserializer::new(context))
        .map_err(Into::into)
}

pub fn from_args<'de, D>() -> Result<D, Error>
where
    D: Deserialize<'de>,
{
    from_args_seed(PhantomData::<D>)
}
