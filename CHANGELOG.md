# Changelog

## Unreleased
### Added
- `from_args()` function to deserialize command line arguments into a type implementing `serde::de::Deserialize`.
- `from_args_seed()` function to deserialize command line arguments into a type implementing `serde::de::DeserializeSeed + Copy`.
- `#[generate]` attribute, with two possible parameters, `doc_item` and `version`, generating help text from documentation and version information respectively.
