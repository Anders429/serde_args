use serde_args_macros::generate;

struct Foo;

#[generate]
impl Foo {}

fn main() {}
