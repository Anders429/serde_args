use serde::Deserialize;
use std::process::exit;

#[derive(Deserialize)]
struct Args {
    foo: bool,
    bar: bool,
    baz: bool,
}

fn main() {
    if let Err(error) = serde_args::from_args::<Args>() {
        println!("{}", error);
        exit(1);
    }
}
