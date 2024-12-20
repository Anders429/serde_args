# serde_args

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Anders429/serde_args/test.yml?branch=master)](https://github.com/Anders429/serde_args/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/serde_args)](https://crates.io/crates/serde_args)
[![docs.rs](https://docs.rs/serde_args/badge.svg)](https://docs.rs/serde_args)
[![License](https://img.shields.io/crates/l/serde_args)](#license)

Command line argument parsing with [`serde`](https://github.com/serde-rs/serde).

This library allows parsing command line arguments into types implementing [`Deserialize`](https://docs.rs/serde/latest/serde/trait.Deserialize.html).

## Features
- Help generation.
- ANSI color support.
- Integration with `serde_derive`, including attributes like `serde(alias)`.

## Usage
Basic usage of `serde_args` simply involves calling the [`from_env()`](https://docs.rs/serde_args/latest/serde_args/fn.from_env.html) function using a type implemented `Deserialize`. The type you provide defines your program's argument format. On success, the type is returned; on failure, a printable [`Error`](https://docs.rs/serde_args/latest/serde_args/struct.Error.html) is returned.

Here is a simple example, created using `serde`'s [`#[derive(Deserialize)]`](https://docs.rs/serde/latest/serde/derive.Deserialize.html) macro:

``` rust
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
#[serde(expecting = "An example program")]
struct Args {
    path: PathBuf,
    #[serde(alias = "f")]
    force: bool,
}

fn main() {
    let args = match serde_args::from_env::<Args>() {
        Ok(args) => args,
        Err(error) => {
            println!("{error}");
            return;
        }
    };
    println!("{args:?}");
}
```

Running the above program with no provided arguments will display the following help output:

```
An example program

USAGE: serde_args.exe [options] <path>

Required Arguments:
  <path>

Global Options:
  -f --force

Override Options:
  -h --help  Display this message.
```

Running the program with example arguments of `README.md -f` will show the parsed arguments:

```
Args { path: "README.md", force: true }
```

## License
This project is licensed under either of

* Apache License, Version 2.0
([LICENSE-APACHE](https://github.com/Anders429/serde_args/blob/HEAD/LICENSE-APACHE) or
http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
([LICENSE-MIT](https://github.com/Anders429/serde_args/blob/HEAD/LICENSE-MIT) or
http://opensource.org/licenses/MIT)

at your option.

### Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
