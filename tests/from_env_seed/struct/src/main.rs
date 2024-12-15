use std::process::exit;
use serde::{Deserialize, de::{Deserializer, DeserializeSeed}};

#[derive(Deserialize)]
struct Args {
    foo: String,
    bar: (),
    baz: i64,
}

struct Seed(i64);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed {
    type Value = Args;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Args::deserialize(deserializer).map(|mut args| {
            args.baz += self.0;
            args
        })
    }
}

fn main() {
    if let Err(error) = serde_args::from_env_seed(&Seed(42)) {
        println!("{}", error);
        exit(1);
    }
}
