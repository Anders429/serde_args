use std::process::exit;
use serde::Deserialize;

#[derive(Deserialize)]
struct Args {
    foo: Option<String>,
    bar: Option<()>,
    baz: Option<i64>,
}

fn main() {
    if let Err(error) = serde_args::from_args::<Args>() {
        println!("{}", error);
        exit(1);
    }
}
