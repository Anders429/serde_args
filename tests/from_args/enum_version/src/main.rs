use serde::Deserialize;
use std::process::exit;

#[serde_args::version]
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

fn main() {
    if let Err(error) = serde_args::from_args::<Command>() {
        println!("{}", error);
        exit(1);
    }
}
