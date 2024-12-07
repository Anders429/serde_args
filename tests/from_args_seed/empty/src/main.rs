use serde::de::{Deserialize, Deserializer, DeserializeSeed};
use std::process::exit;

struct Seed(usize);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed {
    type Value = usize;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        <()>::deserialize(deserializer)?;
        Ok(self.0)
    }
}

fn main() {
    if let Err(error) = serde_args::from_args_seed(&Seed(42)) {
        println!("{}", error);
        exit(1);
    }
}
