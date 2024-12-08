# Changelog

## Unreleased
### Added
- `from_args()` function to deserialize command line arguments into a type implementing `serde::de::Deserialize`.
- `from_args_seed()` function to deserialize command line arguments into a type implementing `serde::de::DeserializeSeed + Copy`.
- `Error` type containing a printable representation of all deserialization errors, including parsing errors, development errors, and help generation.
- `#[generate]` attribute, with two possible parameters, `doc_item` and `version`, generating help text from documentation and version information respectively.
