#![feature(proc_macro_hygiene)]

use serde_args_macros::generate;

#[generate]
mod foo;

fn main() {}
