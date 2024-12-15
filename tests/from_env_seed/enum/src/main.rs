use std::process::exit;
use serde::{Deserialize, de::{Deserializer, DeserializeSeed}};

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    Foo,
    Bar(u8),
    Baz(Option<String>),
    Qux {
        required: String,
        optional: Option<String>,
    },
}

struct Seed(usize);

impl<'a, 'de> DeserializeSeed<'de> for &'a Seed {
    type Value = (usize, Command);

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        Command::deserialize(deserializer).map(|command| (self.0, command))
    }
}

fn main() {
    if let Err(error) = serde_args::from_env_seed(&Seed(42)) {
        println!("{}", error);
        exit(1);
    }
}
