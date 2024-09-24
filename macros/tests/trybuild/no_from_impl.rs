use serde::Deserialize;
use serde_args_macros::help;

#[help]
#[derive(Deserialize)]
#[serde(from = "Qux")]
struct Foo {
    bar: usize,
    baz: String,
}

#[derive(Deserialize)]
struct Qux {
    bar: usize,
    baz: String,
}

fn main() {}
