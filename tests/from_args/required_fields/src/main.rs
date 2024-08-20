use std::process::exit;
use serde::Deserialize;

#[derive(Deserialize)]
struct Args {
    foo: String,
    bar: (),
    baz: i64,
}

fn main() {
    if let Err(error) = serde_args::from_args::<Args>() {
        println!("{}", error);
        exit(1);
    }
}
