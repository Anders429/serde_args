# Changelog

## Unreleased
### Fixed
- Optional fields that are not present will now be provided to the deserializer with a `None` value instead of being ignored completely. This fixes issues with some deserialization patterns involving `Option`s.

## 0.1.0 - 2024-12-15
### Added
- `from_env()` function to deserialize command line arguments into a type implementing `serde::de::Deserialize`.
- `from_env_seed()` function to deserialize command line arguments into a type implementing `serde::de::DeserializeSeed + Copy`.
- `Error` type containing a printable representation of all deserialization errors, including parsing errors, development errors, and help generation.
- `#[generate]` attribute, with two possible parameters, `doc_item` and `version`, generating help text from documentation and version information respectively.
