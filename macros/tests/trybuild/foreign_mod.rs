#![feature(proc_macro_hygiene)]

use serde_args_macros::help;

#[help]
mod foo;

fn main() {}
