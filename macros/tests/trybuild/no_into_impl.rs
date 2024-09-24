use serde::Serialize;
use serde_args_macros::help;

#[help]
#[derive(Clone, Serialize)]
#[serde(into = "Qux")]
struct Foo {
    bar: usize,
    baz: String,
}

#[derive(Serialize)]
struct Qux {
    bar: usize,
    baz: String,
}

fn main() {}
