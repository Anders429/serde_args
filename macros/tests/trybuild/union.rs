use serde_args_macros::help;

#[help]
union Foo {
    bar: usize,
    baz: char,
}

fn main() {}
