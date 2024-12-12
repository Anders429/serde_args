//! A full specification of the format defined by `serde_args`.
//!
//! Command line interfaces created with `serde_args` are entirely defined by the
//! [`serde` data types](https://serde.rs/data-model.html#types) requested by the given type's
//! [`Deserialize`] implementation. The allowed inputs will depend entirely on the methods called
//! on the deserializer.
//!
//! # Primitives
//!
//! Primitives will, in all cases besides units, be parsed from the immediate next non-optional
//! argument string.
//!
//! Note that the rules defined here do not all apply to struct fields. Specifically, booleans are
//! treated differently if called within a struct.
//!
//! ## Units
//!
//! Units, requested with [`Deserializer::deserialize_unit()`],
//! [`Deserializer::deserialize_unit_struct()`], or [`VariantAccess::unit_variant()`], are
//! represented as nothing. No input strings will be parsed.
//!
//! ## Booleans
//!
//! Booleans, requested with [`Deserializer::deserialize_bool()`], can parse either `true` or
//! `false`. Note that these values must be lowercase; `TRUE` or `FALSE` are not accepted, nor are
//! `t`, `f`, `0`, or `1`.
//!
//! ## Numeric Values
//!
//! Numeric values, meaning integers (signed or unsigned) and floats, will be parsed using the
//! corresponding numeric type's [`FromStr`] implementation. For example, calling
//! [`Deserializer::deserialize_u32()`] will result in the next value being parsed with
//! `u32::from_str()`.
//!
//! ## Characters, Strings, and Bytes
//!
//! Characters, strings, and bytes will be parsed by interpreting the next value as the given type.
//! Characters must be exactly one character and strings must be UTF-8. Bytes can be any set of
//! bytes, although there may be limitations regarding what bytes can actually be passed on the
//! command line depending on the operating system being used.
//!
//! # Optionals
//!
//! Optional values, requested with [`Deserializer::deserialize_option()`], will optionally parse
//! the next value *if* it is an optional value. An optional value is preceeded with `--`. If the
//! next value is not optional (or we are at the end of the argument list), the returned value is
//! `None`.
//!
//! For example, parsing an `Option<String>` would interpret the argument `--foo` as `Some("foo")`.
//!
//! # Structs
//!
//! Structs act as the main compound data structure to be used when multiple positional data types
//! are desired.
//!
//! ## Normal Structs
//!
//! Structs have three different types of fields: boolean fields, optional fields, and required
//! fields. The rules for parsing boolean fields and optional fields differ from parsing booleans
//! and optionals as explained above.
//!
//! All boolean and optional fields can be provided at any point and in any order during the
//! parsing of the struct. This means they can be provided at any point between the required fields
//! of the struct, immediately before the first field of the struct, or immediately after the last
//! field of the struct.
//!
//! ### Boolean fields
//! Boolean fields are set to true using the name of the field as an optional value. If the name is
//! never provided, a default value of `false` is used.
//!
//! For example, a boolean field named `foo` could be set to true with the flag `--foo`.
//!
//! ### Optional fields
//!
//! Optional fields are set using the name of the field as an optional value, followed by the value
//! itself. If the name is never provided, a default value of `None` is used.
//!
//! For example, an optional field named `foo` containing a `String` value could be set using
//! `--foo bar`. This would set the field's value to `Some("bar")`.
//!
//! ### Required fields
//!
//! Required fields (sometimes called "positional fields") are all fields that are not booleans or
//! optionals. They must be provided in the order they are defined and cannot be omitted.
//!
//! ## Unit Structs
//!
//! See [Units](#units).
//!
//! ## Newtype Structs
//!
//! Newtype structs are treated as their contained value. `serde_args` does no special wrapping of
//! newtype structs.
//!
//! Note that options provided in `expecting()` on the newtype struct will overwrite the contained
//! type's options. See [`expecting()` Option Specification](#expecting-option-specification) for
//! more details.
//!
//! ## Tuple Structs
//!
//! Tuple structs are not currently supported.
//!
//! # Enums
//!
//! Enums (often called "commands" in terms of command line interfaces) are specified by the
//! identifier parsed from the next available argument (parsed as a string, not as an integer or
//! any other value). The matched variant type will determine how the next arguments are parsed.
//!
//! ## Unit Variants
//!
//! See [Units](#units).
//!
//! ## Struct Variants
//!
//! The next values will be parsed as the provided struct. See [Structs](#structs) for more
//! details.
//!
//! ## Newtype Variants
//!
//! The next values will be parsed as the contained value. See [Newtype Structs](#newtype-structs)
//! for more details.
//!
//! ## Tuple Variants
//!
//! Tuple variants are not currently supported.
//!
//! # Tuples
//!
//! Tuples are not currently supported.
//!
//! # Sequences
//!
//! Sequences are not currently supported.
//!
//! # Maps
//!
//! Maps are not currently supported.
//!
//! # `expecting()` Option Specification
//!
//! While most users will likely want to create types using `serde`'s derive macros, some users may
//! wish to define their [`Deserialize`] implementations by hand. These users will therefore not be
//! able to use the [`#[generate]`](crate::generate) macro to customize the behavior of the
//! deserializer and will need to do so by hand. To do so, the [`Visitor::expecting()`]
//! function in the `Deserialize` implementation must provide that information directly.
//!
//! Note that the following techniques can be combined together to specify multiple customizations
//! on the same container.
//!
//! ## Help Messages
//!
//! There are two types of custom messages that can be provided: container-level messages and
//! field/variant-level messages. These can be defined within the same visitor to allow displaying
//! all kinds of help messages on the same container.
//!
//! ### Container-level messages
//!
//! For `structs` and `enum`s, the container-level message can be provided by simply writing the
//! message to the provided formatter.
//!
//! ```rust
//! use serde::de::Visitor;
//! use std::{
//!     fmt,
//!     fmt::Formatter,
//! };
//!
//! struct ContainerVisitor;
//!
//! impl<'de> Visitor<'de> for ContainerVisitor {
//!     type Value = ();
//!
//!     fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
//!         formatter.write_str("My custom message")
//!     }
//! }
//! ```
//!
//! ### Field and variant messages
//!
//! Custom messages are provided for fields and variants by writing the corresponding message using
//! the field/variant index provided. The field/variant index is provided through
//! `formatter.width()`. Write the corresponding message (and nothing else) when a width is
//! provided corresponding to the field or variant.
//!
//! For example, for a struct defined as follows:
//!
//! ```rust
//! struct Container {
//!     first_field: String,
//!     second_field: Option<usize>,
//! }
//! ```
//!
//! One could specify field messages as follows:
//!
//! ```rust
//! use serde::de::Visitor;
//! use std::{
//!     fmt,
//!     fmt::Formatter,
//! };
//!
//! struct ContainerVisitor;
//!
//! impl<'de> Visitor<'de> for ContainerVisitor {
//!     type Value = ();
//!
//!     fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
//!         match formatter.width() {
//!             Some(0) => formatter.write_str("First field's message"),
//!             Some(1) => formatter.write_str("Second field's message"),
//!             _ => formatter.write_str("Container's message"),
//!         }
//!     }
//! }
//! ```
//!
//! ## Version Information
//!
//! To specify that a `--version` flag should be used, `expecting()` should provide a version to be
//! used as version output when `formatter.fill()` is `'v'`. For example:
//!
//! ```rust
//! use serde::de::Visitor;
//! use std::{
//!     fmt,
//!     fmt::Formatter,
//! };
//!
//! struct VersionVisitor;
//!
//! impl<'de> Visitor<'de> for VersionVisitor {
//!     type Value = ();
//!
//!     fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
//!         if formatter.fill() == 'v' {
//!             formatter.write_str("0.1.0")?;
//!         }
//!         Ok(())
//!     }
//! }
//! ```
//!
//! Note that the version provided to the formatter must be different than what is provided when
//! the version is not requested; otherwise it will be ignored and no `--format` flag will be used.
//!
//! # Unsupported Deserialization Behavior
//!
//! The following behavior is possible to implement in a [`Deserialize`] implementation, but is not
//! supported within `serde_args`.
//!
//! - Not calling a method on the provided [`Deserializer`]
//! - Calling `Deserializer::deserialize_struct()` on one run of `Deserialize::deserialize()`, and
//!   then calling `Deserializer::deserialize_enum()` on the next run (or vice-versa).
//! - Attempting to deserialize an identifier by calling anything besides
//!   `Deserializer::deserialize_identifier()`.
//! - Calling `Deserializer::deserialize_any()` or `Deserializer::deserialize_ignored_any()`.
//!   `serde_args` is **not** a self-describing format.
//!
//! [`Deserialize`]: serde::Deserialize
//! [`Deserializer`]: serde::Deserializer
//! [`Deserializer::deserialize_unit()`]: serde::Deserializer::deserialize_unit()
//! [`Deserializer::deserialize_unit_struct()`]: serde::Deserializer::deserialize_unit_struct()
//! [`Deserializer::deserialize_bool()`]: serde::Deserializer::deserialize_bool()
//! [`Deserializer::deserialize_u32()`]: serde::Deserializer::deserialize_u32()
//! [`Deserializer::deserialize_option()`]: serde::Deserializer::deserialize_option()
//! [`FromStr`]: std::str::FromStr
//! [`VariantAccess::unit_variant()`]: serde::de::VariantAccess::unit_variant()
//! [`Visitor::expecting()`]: serde::de::Visitor::expecting()
