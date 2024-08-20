use claims::{assert_err, assert_ok};
use std::{
    ffi::OsString,
    fmt,
    fmt::{Display, Formatter},
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
