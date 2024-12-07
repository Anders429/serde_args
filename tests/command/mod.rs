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
pub enum Error {
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

pub struct Command {
    pub args: Vec<OsString>,
    pub path: PathBuf,
    pub binary_name: String,
}

impl Command {
    pub fn new<Path>(path: Path) -> Self
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

    pub fn args<Arg, Args>(&mut self, args: Args) -> &mut Self
    where
        Args: IntoIterator<Item = Arg>,
        Arg: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(|arg| arg.into()));
        self
    }

    pub fn run(&self) -> Result<(), Error> {
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

#[macro_export]
macro_rules! assert_run_ok {
    ($command:expr) => {
        ::claims::assert_ok!($command.run());
    };
}

#[macro_export]
macro_rules! assert_run_err {
    ($command:expr, $expected:literal) => {
        let name = $command.binary_name.clone();
        let error = ::claims::assert_err!($command.run());
        if let command::Error::Stdout(stdout) = error {
            assert_eq!(stdout, format!($expected, name = name));
        } else {
            panic!("command failed to execute: {}", error);
        }
    };
}

#[macro_export]
macro_rules! assert_run_err_literal {
    ($command:expr, $expected:literal) => {
        let error = ::claims::assert_err!($command.run());
        if let command::Error::Stdout(stdout) = error {
            assert_eq!(stdout, $expected);
        } else {
            panic!("command failed to execute: {}", error);
        }
    };
}
