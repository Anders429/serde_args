# serde_args_macros

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/Anders429/serde_args/test.yml?branch=master)](https://github.com/Anders429/serde_args/actions/workflows/test.yml)
[![crates.io](https://img.shields.io/crates/v/serde_args_macros)](https://crates.io/crates/serde_args_macros)
[![docs.rs](https://docs.rs/serde_args_macros/badge.svg)](https://docs.rs/serde_args_macros)

Macros for the [`serde_args`](https://github.com/Anders429/serde_args) crate.

Due to its nature as a command line argument parsing format, `serde_args` allows some extra information to be provided to the deserializer. In order to make this process easier, a [`serde`](https://serde.rs/) add-on macro is provided to add additional information to `serde`'s derived `Deserialize` implementation.
