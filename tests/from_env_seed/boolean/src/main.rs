use serde::de::{Deserialize, Deserializer, DeserializeSeed};
use std::process::exit;

struct Seed(usize);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed {
    type Value = Option<usize>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        bool::deserialize(deserializer).map(|b| b.then_some(self.0))
    }
}

fn main() {
    if let Err(error) = serde_args::from_env_seed(&Seed(42)) {
        println!("{}", error);
        exit(1);
    }
}
