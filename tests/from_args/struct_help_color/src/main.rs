use serde::Deserialize;
use std::process::exit;

/// This is a description of my program.
#[serde_args::help]
#[derive(Deserialize)]
struct Args {
    /// Not just any string, but your favorite string.
    foo: String,
    /// This documentation shouldn't show up in the help message.
    bar: (),
    /// Any number other than 9.
    baz: i64,
    /// Determines the quxiness of the program.
    #[serde(alias = "q")]
    qux: Option<u8>,
}

fn main() {
    if let Err(error) = serde_args::from_args::<Args>() {
        println!("{:#}", error);
        exit(1);
    }
}
