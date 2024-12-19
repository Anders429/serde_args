use serde::{Deserialize, Deserializer};
use std::{path::PathBuf, process::exit};

fn optional_default<'de, D>(deserializer: D) -> Result<PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<PathBuf>::deserialize(deserializer)
        .map(|option| option.unwrap_or("default".to_string().into()))
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Args {
    pub input: PathBuf,
    #[serde(deserialize_with = "optional_default")]
    pub directory: PathBuf,
}

fn main() {
    if let Err(error) = serde_args::from_env::<Args>() {
        println!("{}", error);
        exit(1);
    }
}
