//! Ensures that `from_env_seed` behaves correctly.

mod command;

use command::Command;

#[test]
fn empty() {
    assert_run_ok!(Command::new("tests/from_env_seed/empty"));
    assert_run_ok!(Command::new("tests/from_env_seed/empty").args(["--"]));

    assert_run_err!(Command::new("tests/from_env_seed/empty").args(["foo"]), "ERROR: unexpected positional argument: foo\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/empty").args(["--foo"]), "ERROR: unrecognized optional flag: --foo\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/empty").args(["--", "--"]), "ERROR: unexpected positional argument: --\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(
        Command::new("tests/from_env_seed/empty").args(["-h"]),
        "unit\n\nUSAGE: {name} \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/empty").args(["--help"]),
        "unit\n\nUSAGE: {name} \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(Command::new("tests/from_env_seed/empty").args(["--", "-h"]), "ERROR: unexpected positional argument: -h\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/empty").args(["--", "--help"]), "ERROR: unexpected positional argument: --help\n\nUSAGE: {name} \n\nFor more information, use --help.\n");
}

#[test]
fn primitive() {
    assert_run_ok!(Command::new("tests/from_env_seed/primitive").args(["42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/primitive").args(["--", "42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/primitive").args(["42", "--"]));

    assert_run_err!(Command::new("tests/from_env_seed/primitive").args(["foo"]), "ERROR: invalid type: expected u64, found foo\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/primitive").args(["-42"]), "ERROR: invalid type: expected u64, found -42\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n");
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["-h"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["--help"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["42", "--help"]),
        "u64\n\nUSAGE: {name} <u64>\n\nRequired Arguments:\n  <u64>  u64\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["--", "-h"]),
        "ERROR: invalid type: expected u64, found -h\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["--", "--help"]),
        "ERROR: invalid type: expected u64, found --help\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["--", "42", "-h"]),
        "ERROR: unexpected positional argument: -h\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/primitive").args(["--", "42", "--help"]),
        "ERROR: unexpected positional argument: --help\n\nUSAGE: {name} <u64>\n\nFor more information, use --help.\n"
    );
}

#[test]
fn boolean() {
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["true"]));
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["false"]));
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["--", "true"]));
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["--", "false"]));
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["true", "--"]));
    assert_run_ok!(Command::new("tests/from_env_seed/boolean").args(["false", "--"]));

    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["foo"]), "ERROR: invalid type: expected a boolean, found foo\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["0"]), "ERROR: invalid type: expected a boolean, found 0\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["1"]), "ERROR: invalid type: expected a boolean, found 1\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["TRUE"]), "ERROR: invalid type: expected a boolean, found TRUE\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["FALSE"]), "ERROR: invalid type: expected a boolean, found FALSE\n\nUSAGE: {name} <a boolean>\n\nFor more information, use --help.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean"), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["-h"]), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
    assert_run_err!(Command::new("tests/from_env_seed/boolean").args(["--help"]), "a boolean\n\nUSAGE: {name} <a boolean>\n\nRequired Arguments:\n  <a boolean>  a boolean\n\nOverride Options:\n  -h --help  Display this message.\n");
}

#[test]
fn option() {
    assert_run_ok!(Command::new("tests/from_env_seed/option"));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["--"]));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["--foo"]));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["-"]));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["-", "--"]));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["-h"]));
    assert_run_ok!(Command::new("tests/from_env_seed/option").args(["--help"]));

    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--", "--foo"]),
        "ERROR: unrecognized optional flag: --foo\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} [--<a string>]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--", "-"]),
        "ERROR: unrecognized optional flag: -\n\n  tip: a similar option exists: -h\n\nUSAGE: {name} [--<a string>]\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--", "-h"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--", "--help"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--foo", "-h"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/option").args(["--foo", "--help"]),
        "a string\n\nUSAGE: {name} [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
}

#[test]
fn r#struct() {
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["hello", "42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["hello", "-42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["--", "--hello", "42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["--", "--help", "42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["hello", "--", "-42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["hello", "--", "-3"]));
    assert_run_ok!(Command::new("tests/from_env_seed/struct").args(["hello", "42", "--"]));

    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["--"]),
        "ERROR: missing required positional arguments: <foo> <baz>\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello"]),
        "ERROR: missing required positional argument: <baz>\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct"),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["-h"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["--help"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "42", "--help"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["--help", "hello", "42"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "--help", "42"]),
        "struct Args\n\nUSAGE: {name} <foo> <baz>\n\nRequired Arguments:\n  <foo>  \n  <baz>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "42", "hello"]),
        "ERROR: unexpected positional argument: hello\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "42", "--hello"]),
        "ERROR: unrecognized optional flag: --hello\n\n  tip: a similar option exists: --help\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "--", "--help"]),
        "ERROR: invalid type: expected i64, found --help\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/struct").args(["hello", "-3"]),
        "ERROR: unrecognized optional flag: -3\n\n  tip: a similar option exists: -h\n\nUSAGE: {name} <foo> <baz>\n\nFor more information, use --help.\n"
    );
}

#[test]
fn r#enum() {
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["foo"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["bar", "42"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["baz"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["baz", "--"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["baz", "-"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["baz", "--foo"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args(["qux", "hello"]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args([
        "qux",
        "--optional",
        "hi",
        "hello"
    ]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args([
        "qux",
        "hello",
        "--optional",
        "hi"
    ]));
    assert_run_ok!(Command::new("tests/from_env_seed/enum").args([
        "qux",
        "--optional",
        "hi",
        "--",
        "--help"
    ]));

    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["--"]),
        "ERROR: missing required positional argument: <Command>\n\nUSAGE: {name} <Command>\n\nFor more information, use --help.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum"),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["-h"]),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["--help"]),
        "enum Command\n\nUSAGE: {name} <Command>\n\nRequired Arguments:\n  <Command>  enum Command\n\nOverride Options:\n  -h --help  Display this message.\n\nCommand Variants:\n  foo                       \n  bar <u8>                  \n  baz [--<a string>]        \n  qux [options] <required>  \n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["--help", "foo"]),
        "USAGE: {name} foo \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["bar", "--help"]),
        "USAGE: {name} bar <u8>\n\nRequired Arguments:\n  <u8>  u8\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["baz", "--", "--help"]),
        "USAGE: {name} baz [--<a string>]\n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["qux", "--help", "foo"]),
        "USAGE: {name} qux [qux options] <required>\n\nRequired Arguments:\n  <required>  \n\nqux Options:\n  --optional <a string>  \n\nOverride Options:\n  -h --help  Display this message.\n"
    );
    assert_run_err!(
        Command::new("tests/from_env_seed/enum").args(["quux"]),
        "ERROR: unrecognized command: quux\n\n  tip: a similar command exists: qux\n\nUSAGE: {name} <Command>\n\nFor more information, use --help.\n"
    );
}
