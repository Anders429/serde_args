use serde_args_macros::help;

#[help]
struct Foo {
    bar: usize,
    baz: String,
}

fn main() {}
