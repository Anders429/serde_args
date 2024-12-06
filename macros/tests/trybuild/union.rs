use serde_args_macros::generate;

#[generate]
union Foo {
    bar: usize,
    baz: char,
}

fn main() {}
