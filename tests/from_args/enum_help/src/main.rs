use serde::Deserialize;
use std::process::exit;

/// This is a description of my program.
#[serde_args::help]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Command {
    /// Don't provide any arguments to this command.
    Foo,
    /// Provide one argument to this command.
    Bar(u8),
    /// You can do zero or one arguments for this command.
    Baz(Option<String>),
    /// This command takes a required argument and an optional flag.
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
