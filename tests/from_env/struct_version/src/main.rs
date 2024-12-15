use serde::Deserialize;
use std::process::exit;

#[serde_args::generate(version)]
#[derive(Deserialize)]
struct Args {
    foo: String,
    bar: (),
    baz: i64,
    #[serde(alias = "q")]
    qux: Option<u8>,
}

fn main() {
    if let Err(error) = serde_args::from_env::<Args>() {
        println!("{}", error);
        exit(1);
    }
}
