use serde_args_macros::generate;

#[generate(doc_help)]
struct Foo {
    bar: usize,
    baz: String,
}

fn main() {}
