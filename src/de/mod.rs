mod error;

pub use error::Error;

use std::{ffi::OsString, iter::Map};

pub struct Deserializer<Args> {
    args: Args,
}

impl Deserializer<()> {
    pub fn new<Args, Arg>(args: Args) -> Deserializer<Map<Args, impl FnMut(Arg) -> Vec<u8>>>
    where
        Args: Iterator<Item = Arg>,
        Arg: Into<OsString>,
    {
        Deserializer {
            args: args.map(|arg| arg.into().into_encoded_bytes()),
        }
    }
}
