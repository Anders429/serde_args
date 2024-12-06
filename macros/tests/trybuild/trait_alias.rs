use serde_args_macros::generate;

#[generate]
trait Foo = Iterator + Sync;

fn main() {}
