use serde::de::{Deserialize, Deserializer, DeserializeSeed};
use std::process::exit;

struct Seed<'a>(&'a str);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed<'a> {
    type Value = String;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Option::<String>::deserialize(deserializer)?.unwrap_or(self.0.to_owned()))
    }
}

fn main() {
    if let Err(error) = serde_args::from_args_seed(&Seed("foo")) {
        println!("{}", error);
        exit(1);
    }
}
