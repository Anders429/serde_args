use std::process::exit;

fn main() {
    if let Err(error) = serde_args::from_args::<bool>() {
        println!("{}", error);
        exit(1);
    }
}
