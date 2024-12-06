use serde::Serialize;
use serde_args_macros::generate;

#[generate(doc_help)]
#[derive(Serialize)]
struct Foo {
    bar: usize,
    baz: String,
}

fn main() {}
