use serde::Deserialize;
use serde_args_macros::help;

#[help]
#[derive(Deserialize)]
#[serde(from = "=")]
struct Foo {
    bar: usize,
    baz: String,
}

fn main() {}
