//! Command line argument parsing with [`serde`].
//!
//! This library allows parsing command line arguments into types implementing [`Deserialize`].
//! Included are all of the typical features you'd find in an command line argument parsing
//! library, including help generation, help customization, version support, and flexible parsing
//! options.
//!
//! Unlike other argument parsing libraries, `serde_args` uses `serde`'s traits to define the
//! command line interface. No builder or derive interface is provided; instead, any object
//! implementing `Deserialize` can be used. This means that you can use `serde`'s own derive macros
//! (that you are likely already familiar with) or implement the `Deserialize` trait by hand to
//! define your command line interface.
//!
//! `serde_args` defines an unambiguous deserialization format through internal [`Deserializer`]s;
//! therefore it should be noted that not every command line interface can be represented using it.
//! Notably, optional positional parameters are not supported at all, nor are default command
//! values. The format defined here requires that all positional arguments (those *without* a `-`
//! or `--` preceeding them) be required arguments, and that all other arguments be preceeded with
//! either a `-` or `--` (including compound types). See the [format specification](specification)
//! for more details.
//!
//! # Parsing Arguments
//!
//! To use `serde_args` for your own command line argument parsing, you must first define a type
//! implementing `serde`'s [`Deserialize`] trait. This can be done using `serde`'s derive macro or
//! by implementing the trait by hand. For example, a simple program could be defined as:
//!
//! ``` rust
//! use serde::Deserialize;
//! # use serde_derive::Deserialize;
//! use std::path::PathBuf;
//!
//! #[derive(Deserialize)]
//! #[serde(expecting = "An example program")]
//! struct Args {
//!     path: PathBuf,
//!     #[serde(alias = "f")]
//!     force: bool,
//! }
//!
//! fn main() {
//!     let args: Args = match serde_args::from_args() {
//!         Ok(args) => args,
//!         Err(error) => {
//!             println!("{error}");
//!             return;
//!         }
//!     };
//!     // Execute your program with `args`...
//! }
//! ```
//!
//! Command-based interfaces can be defined using `enums`:
//!
//! ``` rust
//! use serde::Deserialize;
//! # use serde_derive::Deserialize;
//! use std::path::PathBuf;
//!
//! #[derive(Deserialize)]
//! #[serde(expecting = "A command-based interface")]
//! #[serde(rename_all = "kebab-case")]
//! enum Command {
//!     Add {
//!         path: PathBuf,
//!     },
//!     Commit {
//!         #[serde(alias = "m")]
//!         message: Option<String>,
//!     },
//!     Push {
//!         #[serde(alias = "f")]
//!         force: bool,
//!     },
//! }
//!
//! fn main() {
//!     let command: Command = match serde_args::from_args() {
//!         Ok(command) => command,
//!         Err(error) => {
//!             println!("{error}");
//!             return;
//!         }
//!     };
//!     // Execute your program with `command`...
//! }
//! ```
//!
//! For simple use cases you can also use existing types that
//! already implement `Deserialize`:
//!
//! ``` rust
//! fn main() {
//!     let value: String = match serde_args::from_args() {
//!         Ok(value) => value,
//!         Err(error) => {
//!             println!("{error}");
//!             return;
//!         }
//!     };
//!     // Execute your program with `value`...
//! }
//! ```
//!
//! Note that the only way to deserialize using this crate is through [`from_args()`] and
//! [`from_args_seed()`]. No public [`Deserializer`] is provided.
//!
//! # Error Formatting
//!
//! On failure, [`from_args()`] will return an [`Error`]. This will occur when the provided type is
//! incompatible with this crate (see [Supported `serde` Attributes](#supported-serde-attributes)
//! for common reasons why types are not compatible), when the user has input command line
//! arguments that cannot be parsed into the provided type, or when the user requests the generated
//! help message (either through the `--help` flag or by providing no arguments). In any case, the
//! returned `Error` implements the [`Display`] trait and is able to be printed and displayed to
//! the user.
//!
//! For example, a program taking a single unsigned integer value as a parameter would print an
//! error that occurred like so:
//!
//! ```rust
//! if let Err(error) = serde_args::from_args::<usize>() {
//!     println!("{error}");
//! }
//! ```
//!
//! To print an error that is formatted with ANSI color sequences, use the "alternate" form of
//! printing with the `#` flag:
//!
//! ```rust
//! if let Err(error) = serde_args::from_args::<usize>() {
//!     println!("{error:#}");
//! }
//! ```
//!
//! # Customization
//!
//! `serde_args` allows for customizing in the form of messages to be displayed in help output and
//! the displaying of version information. These are most easily customized using the
//! [`#[generate]`](generate) attribute (requires the `macros` feature`). This macro must be
//! combined with `serde`'s `Deserialize` derive macro.
//!
//! ## Custom Help Messages
//!
//! Descriptions for each of your fields or variants can be automatically imported from your struct
//! or enum's doc comment using `#[generate]` with `doc_help` as a parameter:
//!
//! ``` rust
//! use serde::Deserialize;
//! # use serde_derive::Deserialize;
//! use std::path::PathBuf;
//!
//! /// An example program.
//! #[derive(Deserialize)]
//! #[serde_args::generate(doc_help)]
//! struct Args {
//!     /// The path to operate on.
//!     path: PathBuf,
//!     /// Whether the program's behavior should be forced.
//!     #[serde(alias = "f")]
//!     force: bool,
//! }
//!
//! fn main() {
//!     let args: Args = match serde_args::from_args() {
//!         Ok(args) => args,
//!         Err(error) => {
//!             println!("{error}");
//!             return;
//!         }
//!     };
//!     // Execute your program with `args`...
//! }
//! ```
//!
//! ## Version Information
//!
//! To automatically make the version of your crate available through a `--version` flag, use
//! `#[generate]` with `version` as a parameter:
//!
//! ```rust
//! use serde::Deserialize;
//! # use serde_derive::Deserialize;
//!
//! #[derive(Deserialize)]
//! #[serde_args::generate(version)]
//! struct Args {
//!     foo: String,
//!     bar: bool,
//! }
//!
//! fn main() {
//!     let args: Args = match serde_args::from_args() {
//!         Ok(args) => args,
//!         Err(error) => {
//!             println!("{error}");
//!             return;
//!         }
//!     };
//!     // Execute your program with `args`...
//! }
//! ```
//!
//! ## Customization Without Deriving
//!
//! To provide these customization options without deriving, see
//! [`expecting()` Option Specification](specification/index.html#expecting-option-specification).
//!
//! # Supported `serde` Attributes
//!
//! Nearly all `serde` attributes are supported. Those that are not supported are those that
//! require a self-describing deserializer (the format defined by `serde_args` is **not**
//! self-describing). Specifically, the following attributes will not work:
//!
//! - [`#[serde(flatten)]`](https://serde.rs/field-attrs.html#flatten)
//! - [`#[serde(tag = "type")]`](https://serde.rs/container-attrs.html#tag) - Doesn't work for
//! enums, but it will work for structs.
//! - [`#[serde(tag = "t", content = "c")]`](https://serde.rs/container-attrs.html#tag--content)
//! - [`#[serde(untagged)]`](https://serde.rs/container-attrs.html#untagged) - Not allowed on enums
//! or on variants.
//! - [`#[serde(other)]`](https://serde.rs/variant-attrs.html#other)
//!
//! Aside from the above list, all other attributes are supported. Some attributes are especially
//! useful for defining command line interfaces, including:
//!
//! - [`#[serde(alias)]`](https://serde.rs/field-attrs.html#alias) - Useful for defining multiple
//! names for optional fields or command variants.
//! - [`#[serde(expecting)]`](https://serde.rs/container-attrs.html#expecting) - Can be used to
//! define a description for your program. Whatever is provided here will be output at the top of
//! the generated help message.
//!   - Note that many users will want to use [`#[serde_args::generate(doc_help)]`](generate) to
//!     automatically populate this message from the container's doc comment instead.
//! - [`#[serde(rename_all)]`](https://serde.rs/container-attrs.html#rename_all) - Useful for
//! renaming all field names or enum variants to kebab-case, which is common for command-line
//! tools.
//!
//! [`Deserializer`]: serde::Deserializer
//! [`Display`]: std::fmt::Display

pub mod specification;

mod de;
mod error;
mod key;
mod parse;
mod trace;

pub use error::Error;
#[cfg(feature = "macros")]
pub use serde_args_macros::generate;

use de::Deserializer;
use parse::parse;
use serde::de::{
    Deserialize,
    DeserializeSeed,
};
use std::{
    env,
    ffi::OsString,
    marker::PhantomData,
    path::PathBuf,
};
use trace::trace;

pub fn from_args_seed<'de, D>(seed: D) -> Result<D::Value, Error>
where
    D: Copy + DeserializeSeed<'de>,
{
    let mut shape = trace(seed)?;

    let mut args = env::args_os();
    let executable_path: OsString = {
        let path_str = args.next().expect("could not obtain binary name");
        let path_buf = PathBuf::from(&path_str);
        if let Some(file_name) = path_buf.file_name() {
            file_name.to_owned()
        } else {
            path_str
        }
    };

    let context = match parse(args, &mut shape) {
        Ok(context) => context,
        Err(error) => return Err(Error::from_parsing_error(error, executable_path, shape)),
    };

    seed.deserialize(Deserializer::new(context))
        .map_err(|error| Error::from_deserializing_error(error, executable_path, shape))
}

pub fn from_args<'de, D>() -> Result<D, Error>
where
    D: Deserialize<'de>,
{
    from_args_seed(PhantomData::<D>)
}
