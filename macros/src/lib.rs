//! Macros for the [`serde_args`](https://docs.rs/serde_args/latest/serde_args) crate.
//!
//! Due to its nature as a command line argument parsing format, `serde_args` allows some extra
//! information to be provided to the deserializer. In order to make this process easier, a `serde`
//! add-on macro is provided in the form of [`macro@generate`]. This attribute modifies the
//! [`Deserialize`] implementation provided by serde's derive macro to include additional
//! information relevant for `serde_args`.
//!
//! [`Deserialize`]: https://docs.rs/serde/latest/serde/derive.Deserialize.html

mod attributes;
mod container;
mod generate;
mod help;
#[cfg(test)]
mod test;
mod version;

use container::Container;
use proc_macro::TokenStream;

/// Add on for
/// [`#[derive(Deserialize)]`](https://docs.rs/serde/latest/serde/derive.Deserialize.html) to add
/// `serde_args`-specific information.
///
/// This attribute modifies an existing derived `Deserialize` implementation to include extra
/// information specific to `serde_args`. Specifically, it can generate help messages from doc
/// comments and version information from the crate's metadata.
///
/// `serde_args::generate` can take any of the following parameters:
///
/// - `doc_help`
/// - `version`
///
/// `doc_help` will generate help messages for the container, along with its fields/variants, using
/// the item's doc comments. For example, using doc help on the following struct:
///
/// ``` rust
/// use serde::Deserialize;
/// use std::path::PathBuf;
/// # mod serde_args {
/// #     pub use serde_args_macros::generate;
/// # }
///
/// /// An example program.
/// #[derive(Deserialize)]
/// #[serde_args::generate(doc_help)]
/// struct Args {
///     /// The file to be operated on.
///     file: PathBuf,
///     /// Whether the program's behavior should be forced.
///     #[serde(alias = "f")]
///     force: bool,
/// }
/// #
/// # fn main() {}
/// ```
///
/// will generate help messages (to be displayed when `--help` is requested) for the container and
/// each of the fields with the messages "An example program.", "The file to be operated on.", and
/// "Whether the program's behavior should be forced."
///
/// `version` will activate the `--version` optional flag and include your crate's version,
/// extracted from your `Cargo.toml`'s `version` field. For example, it can be enabled by:
///
/// ``` rust
/// use serde::Deserialize;
/// use std::path::PathBuf;
///
/// #[derive(Deserialize)]
/// #[serde_args_macros::generate(version)]
/// struct Args {
///     file: PathBuf,
///     #[serde(alias = "f")]
///     force: bool,
/// }
/// #
/// # fn main() {}
/// ```
///
/// These parameters can also be combined. `#[serde_args::generate(version, doc_help)]` will
/// generate both results on the same container.
///
/// Note that this attribute will wrap the serialized/deserialized type in a newtype. This has no
/// effect on `serde_args`, but it could affect other formats if the same type is used across
/// multiple formats.
#[proc_macro_attribute]
pub fn generate(attr: TokenStream, item: TokenStream) -> TokenStream {
    generate::process(attr.into(), item.into()).into()
}
