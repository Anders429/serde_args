use serde::Deserialize;
use std::process::exit;

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    #[serde(alias = "f")]
    Foo,
    Bar(u8),
    #[serde(alias = "b")]
    Baz(Option<String>),
    #[serde(alias = "q")]
    Qux {
        required: String,
        optional: Option<String>,
    },
}

fn main() {
    if let Err(error) = serde_args::from_env::<Command>() {
        println!("{}", error);
        exit(1);
    }
}
