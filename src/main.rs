// #![allow(dead_code)]

// use serde_derive::Deserialize;
// use std::num::NonZeroU8;

// /// Some documentation.
// ///
// /// Even with multiple lines.
// #[serde_args_macros::help]
// #[derive(Debug, Deserialize)]
// #[serde(expecting = "This is a description of my application.")]
// struct Args {
//     /// Command Documentation
//     command: Command,
//     /// Global Option documentation.
//     ///
//     /// It has multiple lines and an alias.
//     #[serde(alias = "global-opt")]
//     global_opt: Option<()>,
//     #[serde(alias = "non-zero-opt")]
//     non_zero_opt: Option<NonZeroU8>,
//     /// Some documentation for this short flag.
//     c: Option<()>,
// }

// #[serde_args_macros::help]
// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "kebab-case")]
// enum Command {
//     /// Clone a git repository.
//     Clone { remote: String, help: Option<()> },
//     /// Show a diff or something.
//     Diff {
//         base: Option<String>,
//         head: Option<String>,
//         path: Option<String>,
//         color: Option<ColorWhen>,
//     },
//     /// You'll use this one a lot. Good luck.
//     Commit {
//         #[serde(alias = "m")]
//         #[serde(alias = "m")]
//         #[serde(alias = "❤")]
//         message: Option<String>,
//     },
//     /// Use this to break production.
//     Push { remote: String, refspec: String },
//     /// A test command for seeing how newtype variants work.
//     Test(Option<()>),
// }

// #[derive(Debug, Deserialize)]
// #[serde(rename_all = "kebab-case")]
// enum ColorWhen {
//     Always,
//     Auto,
//     Never,
// }

// fn main() {
//     match serde_args::from_args::<Args>() {
//         Ok(args) => {
//             dbg!(args);
//         }
//         Err(error) => {
//             println!("{}", error);
//         }
//     };
// }

fn main() {}
