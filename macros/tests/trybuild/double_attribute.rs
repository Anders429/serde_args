use serde::Deserialize;
use serde_args_macros::generate;

#[generate(version)]
#[generate(version)]
#[derive(Deserialize)]
struct Struct {
    foo: u32,
    bar: String,
}

fn main() {}