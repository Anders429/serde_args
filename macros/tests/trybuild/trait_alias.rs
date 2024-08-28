use serde_args_macros::help;

#[help]
trait Foo = Iterator + Sync;

fn main() {}
