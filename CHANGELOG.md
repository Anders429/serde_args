# Changelog

## Unreleased
### Added
- `from_args()` function to deserialize command line arguments into a type implementing `serde::de::Deserialize`.
- `from_args_seed()` function to deserialize command line arguments into a type implementing `serde::de::DeserializeSeed + Copy`.
- `#[help]` attribute for generating help text using a struct or enum's documentation.
