use serde::Serialize;
use serde_args_macros::help;

#[help]
#[derive(Serialize)]
struct Foo {
    bar: usize,
    baz: String,
}

fn main() {}
