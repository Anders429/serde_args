use std::process::exit;

fn main() {
    if let Err(error) = serde_args::from_env::<Option<String>>() {
        println!("{}", error);
        exit(1);
    }
}
