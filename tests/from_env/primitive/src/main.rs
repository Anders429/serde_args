use std::process::exit;

fn main() {
    if let Err(error) = serde_args::from_env::<u64>() {
        println!("{}", error);
        exit(1);
    }
}
