mod de;
mod error;
mod key;
mod parse;
mod trace;

pub use error::Error;
#[cfg(feature = "macros")]
pub use serde_args_macros::generate;

use de::Deserializer;
use parse::parse;
use serde::de::{
    Deserialize,
    DeserializeSeed,
};
use std::{
    env,
    ffi::OsString,
    marker::PhantomData,
    path::PathBuf,
};
use trace::trace;

pub fn from_args_seed<'de, D>(seed: D) -> Result<D::Value, Error>
where
    D: Copy + DeserializeSeed<'de>,
{
    let mut shape = trace(seed)?;

    let mut args = env::args_os();
    let executable_path: OsString = {
        let path_str = args.next().expect("could not obtain binary name");
        let path_buf = PathBuf::from(&path_str);
        if let Some(file_name) = path_buf.file_name() {
            file_name.to_owned()
        } else {
            path_str
        }
    };

    let context = match parse(args, &mut shape) {
        Ok(context) => context,
        Err(error) => return Err(Error::from_parsing_error(error, executable_path, shape)),
    };

    seed.deserialize(Deserializer::new(context))
        .map_err(|error| Error::from_deserializing_error(error, executable_path, shape))
}

pub fn from_args<'de, D>() -> Result<D, Error>
where
    D: Deserialize<'de>,
{
    from_args_seed(PhantomData::<D>)
}
