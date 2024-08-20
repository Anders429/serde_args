use std::process::exit;

fn main() {
    if let Err(error) = serde_args::from_args::<u64>() {
        println!("{}", error);
        exit(1);
    }
}
