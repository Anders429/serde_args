use serde::de::{
    Deserialize,
    DeserializeSeed,
    Deserializer,
};
use std::process::exit;

struct Seed(u64);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed {
    type Value = u64;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(u64::deserialize(deserializer)? + self.0)
    }
}

fn main() {
    if let Err(error) = serde_args::from_args_seed(&Seed(42)) {
        println!("{}", error);
        exit(1);
    }
}
