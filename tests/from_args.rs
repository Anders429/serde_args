use claims::{
    assert_err,
    assert_ok,
};
use std::{
    ffi::OsString,
    fmt,
    fmt::{
        Display,
        Formatter,
    },
    io,
    io::Read,
    path::PathBuf,
    process,
    process::Stdio,
};

#[derive(Debug)]
enum Error {
    IO(io::Error),
    Stdout(String),
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::IO(error)
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Stdout(error)
    }
}

impl Display for Error {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(error) => write!(formatter, "IO error: {}", error),
            Self::Stdout(error) => write!(
                formatter,
                "failed with nonzero exit code; captured stdout: {}",
                error
            ),
        }
    }
}

struct Command {
    args: Vec<OsString>,
    path: PathBuf,
    binary_name: String,
}

impl Command {
    fn new<Path>(path: Path) -> Self
    where
        Path: Into<PathBuf>,
    {
        let path = path.into();
        Self {
            args: Vec::new(),
            #[cfg(not(target_os = "windows"))]
            binary_name: path.file_name().unwrap().to_string_lossy().into(),
            #[cfg(target_os = "windows")]
            binary_name: format!("{}.exe", path.file_name().unwrap().to_string_lossy()),
            path,
        }
    }

    fn args<Arg, Args>(&mut self, args: Args) -> &mut Self
    where
        Args: IntoIterator<Item = Arg>,
        Arg: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(|arg| arg.into()));
        self
    }

    fn run(&self) -> Result<(), Error> {
        let mut program = process::Command::new("cargo")
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .args(["run", "--"])
            .args(&self.args)
            .current_dir(&self.path)
            .spawn()?;
        if program.wait()?.success() {
            Ok(())
        } else {
            let mut error = String::new();
            program
                .stdout
                .ok_or(io::Error::other("no stdout"))?
                .read_to_string(&mut error)?;
            return Err(Error::Stdout(error));
        }
    }
}

macro_rules! assert_run_ok {
    ($command:expr) => {
        assert_ok!($command.run());
    };
}

macro_rules! assert_run_err {
    ($command:expr, $expected:literal) => {
        let name = $command.binary_name.clone();
        let error = assert_err!($command.run());
        if let Error::Stdout(stdout) = error {
            assert_eq!(stdout, format!($expected, name = name));
        } else {
            panic!("command failed to execute: {}", error);
        }
    };
}

macro_rules! assert_run_err_literal {
    ($command:expr, $expected:literal) => {
        let error = assert_err!($command.run());
        if let Error::Stdout(stdout) = error {
            assert_eq!(stdout, $expected);
        } else {
            panic!("command failed to execute: {}", error);
        }
    };
}

#[test]
fn empty() {
    assert_run_ok!(Command::new("tests/from_args/empty"));
    assert_run_ok!(Command::new("tests/from_args/empty").args(["--"]));

    assert_run_err!(Command::new("tests/from_args/empty").args(["foo"]), "ERROR: unexpected positional argument: foo\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/empty").args(["--foo"]), "ERROR: unrecognized optional flag: --foo\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/empty").args(["--", "--"]), "ERROR: unexpected positional argument: --\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(
        Command::new("tests/from_args/empty").args(["-h"]),
        "unit\n\nUSAGE: {name} \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/empty").args(["--help"]),
        "unit\n\nUSAGE: {name} \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(Command::new("tests/from_args/empty").args(["--", "-h"]), "ERROR: unexpected positional argument: -h\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/empty").args(["--", "--help"]), "ERROR: unexpected positional argument: --help\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
}

#[test]
fn primitive() {
    assert_run_ok!(Command::new("tests/from_args/primitive").args(["42"]));
    assert_run_ok!(Command::new("tests/from_args/primitive").args(["--", "42"]));
    assert_run_ok!(Command::new("tests/from_args/primitive").args(["42", "--"]));

    assert_run_err!(Command::new("tests/from_args/primitive").args(["foo"]), "ERROR: invalid type: expected u64, found foo\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/primitive").args(["-42"]), "ERROR: invalid type: expected u64, found -42\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n");
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["-h"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["--help"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["42", "--help"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["--", "-h"]),
        "ERROR: invalid type: expected u64, found -h\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["--", "--help"]),
        "ERROR: invalid type: expected u64, found --help\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["--", "42", "-h"]),
        "ERROR: unexpected positional argument: -h\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/primitive").args(["--", "42", "--help"]),
        "ERROR: unexpected positional argument: --help\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
}

#[test]
fn boolean() {
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["true"]));
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["false"]));
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["--", "true"]));
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["--", "false"]));
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["true", "--"]));
    assert_run_ok!(Command::new("tests/from_args/boolean").args(["false", "--"]));

    assert_run_err!(Command::new("tests/from_args/boolean").args(["foo"]), "ERROR: invalid type: expected a boolean, found foo\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["0"]), "ERROR: invalid type: expected a boolean, found 0\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["1"]), "ERROR: invalid type: expected a boolean, found 1\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["TRUE"]), "ERROR: invalid type: expected a boolean, found TRUE\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["FALSE"]), "ERROR: invalid type: expected a boolean, found FALSE\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_args/boolean"), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["-h"]), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
    assert_run_err!(Command::new("tests/from_args/boolean").args(["--help"]), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
}

#[test]
fn option() {
    assert_run_ok!(Command::new("tests/from_args/option"));
    assert_run_ok!(Command::new("tests/from_args/option").args(["--"]));
    assert_run_ok!(Command::new("tests/from_args/option").args(["--foo"]));
    assert_run_ok!(Command::new("tests/from_args/option").args(["-"]));
    assert_run_ok!(Command::new("tests/from_args/option").args(["-", "--"]));
    assert_run_ok!(Command::new("tests/from_args/option").args(["-h"]));
    assert_run_ok!(Command::new("tests/from_args/option").args(["--help"]));

    assert_run_err!(
        Command::new("tests/from_args/option").args(["--", "--foo"]),
        "ERROR: unrecognized optional flag: --foo\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} [--<a string>]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/option").args(["--", "-"]),
        "ERROR: unrecognized optional flag: -\n\n  tip: a similar option exists: -h\n\nUSAGE: {name} [--<a string>]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/option").args(["--", "-h"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/option").args(["--", "--help"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/option").args(["--foo", "-h"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/option").args(["--foo", "--help"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
}

#[test]
fn required_fields() {
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["hello", "42"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["hello", "-42"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["--", "--hello", "42"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["--", "--help", "42"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["hello", "--", "-42"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["hello", "--", "-3"]));
    assert_run_ok!(Command::new("tests/from_args/required_fields").args(["hello", "42", "--"]));

    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["--"]),
        "ERROR: missing required positional arguments: <foo> <baz>\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello"]),
        "ERROR: missing required positional argument: <baz>\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields"),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["-h"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["--help"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "42", "--help"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["--help", "hello", "42"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "--help", "42"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "42", "hello"]),
        "ERROR: unexpected positional argument: hello\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "42", "--hello"]),
        "ERROR: unrecognized optional flag: --hello\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "--", "--help"]),
        "ERROR: invalid type: expected i64, found --help\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/required_fields").args(["hello", "-3"]),
        "ERROR: unrecognized optional flag: -3\n\n  tip: a similar option exists: -h\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
}

#[test]
fn optional_fields() {
    assert_run_ok!(Command::new("tests/from_args/optional_fields"));
    assert_run_ok!(Command::new("tests/from_args/optional_fields").args(["--foo", "hello"]));
    assert_run_ok!(Command::new("tests/from_args/optional_fields").args(["--bar"]));
    assert_run_ok!(Command::new("tests/from_args/optional_fields").args(["--baz", "42"]));
    assert_run_ok!(Command::new("tests/from_args/optional_fields")
        .args(["--foo", "hello", "--baz", "42", "--bar"]));
    assert_run_ok!(Command::new("tests/from_args/optional_fields")
        .args(["--foo", "hello", "--baz", "--", "-3",]));

    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--qux"]),
        "ERROR: unrecognized optional flag: --qux\n\n  tip: a similar option exists: --foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--foo"]),
        "ERROR: missing required positional argument: <a string>\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--baz"]),
        "ERROR: missing required positional argument: <i64>\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--bar", "--", "--foo"]),
        "ERROR: unexpected positional argument: --foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--foo", "hello", "--baz", "42", "--bar", "--help"]),
        "struct Args\n\nUSAGE: {name} [options]\n\nGlobal Options:\n  --foo <a string>  \n  --bar             \n  --baz <i64>       \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--foo", "hello", "--baz", "-h", "42", "--bar"]),
        "struct Args\n\nUSAGE: {name} [options]\n\nGlobal Options:\n  --foo <a string>  \n  --bar             \n  --baz <i64>       \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["foo"]),
        "ERROR: unexpected positional argument: foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/optional_fields").args(["--", "--foo"]),
        "ERROR: unexpected positional argument: --foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
}

#[test]
fn boolean_fields() {
    assert_run_ok!(Command::new("tests/from_args/boolean_fields"));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--foo"]));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--bar"]));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--baz"]));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--bar", "--foo"]));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--bar", "--baz"]));
    assert_run_ok!(Command::new("tests/from_args/boolean_fields").args(["--baz", "--bar", "--foo"]));

    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--qux"]),
        "ERROR: unrecognized optional flag: --qux\n\n  tip: a similar option exists: --foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--foo", "--foo"]),
        "ERROR: the argument --foo cannot be used multiple times\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--foo", "true"]),
        "ERROR: unexpected positional argument: true\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--", "--foo"]),
        "ERROR: unexpected positional argument: --foo\n\nUSAGE: {name} [options]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--help"]),
        "struct Args\n\nUSAGE: {name} [options]\n\nGlobal Options:\n  --foo   \n  --bar   \n  --baz   \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--baz", "--help"]),
        "struct Args\n\nUSAGE: {name} [options]\n\nGlobal Options:\n  --foo   \n  --bar   \n  --baz   \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/boolean_fields").args(["--help", "--bar"]),
        "struct Args\n\nUSAGE: {name} [options]\n\nGlobal Options:\n  --foo   \n  --bar   \n  --baz   \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
}

#[test]
fn r#enum() {
    assert_run_ok!(Command::new("tests/from_args/enum").args(["foo"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["bar", "42"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["baz"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["baz", "--"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["baz", "-"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["baz", "--foo"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["qux", "hello"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["qux", "--optional", "hi", "hello"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args(["qux", "hello", "--optional", "hi"]));
    assert_run_ok!(Command::new("tests/from_args/enum").args([
        "qux",
        "--optional",
        "hi",
        "--",
        "--help"
    ]));

    assert_run_err!(
        Command::new("tests/from_args/enum").args(["--"]),
        "ERROR: missing required positional argument: <Command>\n\nUSAGE: {name} <Command>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum"),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["-h"]),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["--help"]),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["--help", "foo"]),
        "USAGE: {name} foo \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["bar", "--help"]),
        "USAGE: {name} bar <u8>\n\nRequired Arguments:\n  <u8>  u8\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["baz", "--", "--help"]),
        "USAGE: {name} baz [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["qux", "--help", "foo"]),
        "USAGE: {name} qux [qux options] <required>\n\nRequired Arguments:\n  <required>  \n\nqux Options:\n  --optional <a string>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_args/enum").args(["quux"]),
        "ERROR: unrecognized command: quux\n\n  tip: a similar command exists: qux\n\nUSAGE: {name} <Command>\n\nFor more information, use --help.\n"
    );
}

#[test]
fn struct_help() {
    assert_run_err!(
        Command::new("tests/from_args/struct_help").args(["--help"]),
        "This is a description of my program.\n\nUSAGE: {name} [options] <foo> <baz>\n\nRequired Arguments:\n  <foo>  Not just any string, but your favorite string.\n  <baz>  Any number other than 9.\n\nGlobal Options:\n  -q --qux <u8>  Determines the quxiness of the program.\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
}

#[test]
fn enum_help() {
    assert_run_err!(
        Command::new("tests/from_args/enum_help").args(["--help"]),
        "This is a description of my program.\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  This is a description of my program.\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       Don't provide any arguments to this command.\n  bar <u8>                  Provide one argument to this command.\n  baz [--<a string>]        You can do zero or one arguments for this command.\n  qux [options] <required>  This command takes a required argument and an optional flag.\n"
    );
}

#[test]
fn struct_help_color() {
    assert_run_err!(
        Command::new("tests/from_args/struct_help_color").args(["--help"]),
        "This is a description of my program.\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96m{name}\x1b[0m \x1b[36m[options] <foo> <baz>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<foo>\x1b[0m  Not just any string, but your favorite string.\n  \x1b[96m<baz>\x1b[0m  Any number other than 9.\n\n\x1b[97mGlobal Options:\x1b[0m\n  \x1b[96m-q\x1b[0m \x1b[96m--qux\x1b[0m \x1b[36m<u8>\x1b[0m  Determines the quxiness of the program.\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message.\n"
    );
}

#[test]
fn enum_help_color() {
    assert_run_err!(
        Command::new("tests/from_args/enum_help_color").args(["--help"]),
        "This is a description of my program.\n\n\x1b[97mUSAGE\x1b[0m: \x1b[96m{name}\x1b[0m \x1b[36m<Command>\x1b[0m\n\n\x1b[97mRequired Arguments:\x1b[0m\n  \x1b[96m<Command>\x1b[0m  This is a description of my program.\n\n\x1b[97mOverride Options:\x1b[0m\n  \x1b[96m-h --help\x1b[0m  Display this message.\n\n\x1b[97mCommand Variants:\x1b[0m\n  \x1b[96mfoo \x1b[0m\x1b[36m\x1b[0m                      Don't provide any arguments to this command.\n  \x1b[96mbar \x1b[0m\x1b[36m<u8>\x1b[0m                  Provide one argument to this command.\n  \x1b[96mbaz \x1b[0m\x1b[36m[--<a string>]\x1b[0m        You can do zero or one arguments for this command.\n  \x1b[96mqux \x1b[0m\x1b[36m[options] <required>\x1b[0m  This command takes a required argument and an optional flag.\n"
    );
}

#[test]
fn struct_version() {
    assert_run_err_literal!(
        Command::new("tests/from_args/struct_version").args(["--version"]),
        "0.0.0\n"
    );
}

#[test]
fn enum_version() {
    assert_run_err_literal!(
        Command::new("tests/from_args/enum_version").args(["--version"]),
        "0.0.0\n"
    );
}

#[test]
fn struct_version_help() {
    assert_run_err_literal!(
        Command::new("tests/from_args/struct_version_help").args(["--version"]),
        "0.0.0\n"
    );
}

#[test]
fn enum_version_help() {
    assert_run_err_literal!(
        Command::new("tests/from_args/enum_version_help").args(["--version"]),
        "0.0.0\n"
    );
}
