use std::env::{var, var_os};
use std::ffi::OsStr;
use std::fmt;
use std::fs::copy;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};
use std::str::Utf8Error;

fn main() -> Result<(), Error> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-env-changed=CARGO_ENCODED_RUSTFLAGS");
    println!("cargo::rerun-if-env-changed=OUT_DIR");
    println!("cargo::rerun-if-env-changed=RUSTC");
    println!("cargo::rerun-if-env-changed=TARGET");

    if cfg!(not(any(unix, windows))) {
        return Ok(());
    }

    let out_dir = env_path("OUT_DIR")?;
    let rustc = env_path("RUSTC")?;
    let target = env_path("TARGET")?;

    let mut dest_lib_dir = out_dir.as_path();
    if !out_dir.join("deps").is_dir() {
        for _ in 0..3 {
            match dest_lib_dir.parent() {
                Some(p) => dest_lib_dir = p,
                None => return Err(Error::OutdirParents(out_dir)),
            }
        }
    }

    let mut command = Command::new(rustc);
    command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .arg("--print=target-libdir")
        .arg("--print=cfg")
        .arg("--target")
        .arg(target);
    if let Ok(args) = var("CARGO_ENCODED_RUSTFLAGS") {
        for arg in args.split('\x1f') {
            command.arg(arg);
        }
    }
    let output = command.output().map_err(Error::RustcRun)?;
    if !output.status.success() {
        return Err(Error::RustcExit(output.status));
    }
    let output = { output }.stdout;
    let output = std::str::from_utf8(&output).map_err(Error::RustcUtf8)?;
    let mut output = output.lines();

    let lib_path = output.next().map(Path::new).ok_or(Error::NoTargetLibdir)?;
    let mut lib_ext = "so";
    for line in output {
        lib_ext = match line {
            r#"target_os="linux""# => break,
            r#"target_vendor="apple""# => "dylib",
            r#"target_os="windows""# => "dll",
            _ => continue,
        };
        break;
    }

    for lib in lib_path
        .read_dir()
        .map_err(|err| Error::ReadDir(err, lib_path.to_owned()))?
    {
        let Ok(lib) = lib else {
            continue;
        };

        let file_name = PathBuf::from(lib.file_name());
        if file_name.extension() != Some(OsStr::new(lib_ext)) {
            continue;
        }
        let Some(str_filename) = file_name.to_str() else {
            continue;
        };
        if !str_filename
            .strip_prefix("lib")
            .unwrap_or(str_filename)
            .starts_with("std-")
        {
            continue;
        }

        let dest = dest_lib_dir.join(&file_name);
        if let Ok(metadata) = dest.symlink_metadata() {
            if metadata.is_dir() || metadata.is_symlink() {
                return Ok(());
            }
        }

        let src = lib_path.join(file_name);

        #[cfg(unix)]
        if std::os::unix::fs::symlink(&src, &dest).is_ok() {
            return Ok(());
        }

        copy(&src, &dest).map_err(|err| Error::Copy(err, src, dest))?;

        return Ok(());
    }
    Err(Error::NotFound(lib_path.to_owned(), lib_ext))
}

fn env_path(key: &'static str) -> Result<PathBuf, Error> {
    var_os(key).map(PathBuf::from).ok_or(Error::EnvVar(key))
}

enum Error {
    EnvVar(&'static str),
    OutdirParents(PathBuf),
    RustcRun(std::io::Error),
    RustcExit(ExitStatus),
    RustcUtf8(Utf8Error),
    NoTargetLibdir,
    ReadDir(std::io::Error, PathBuf),
    Copy(std::io::Error, PathBuf, PathBuf),
    NotFound(PathBuf, &'static str),
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::EnvVar(_)
            | Error::OutdirParents(_)
            | Error::RustcExit(_)
            | Error::NoTargetLibdir
            | Error::NotFound(_, _) => None,
            Error::RustcUtf8(err) => Some(err),
            Error::RustcRun(err) | Error::ReadDir(err, _) | Error::Copy(err, _, _) => Some(err),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvVar(path) => {
                write!(
                    f,
                    "Environment variable `{}` is unset.",
                    path.escape_debug(),
                )
            }
            Self::OutdirParents(path) => {
                write!(f, "Unexpected `OUT_DIR` layout `{}`.", path.display())
            }
            Self::RustcRun(err) => {
                writeln!(f, "Could not run `target-libdir` and `cfg` query.\n{err}")
            }
            Self::RustcExit(status) => {
                write!(f, "Query for `target-libdir` and `cfg` returned {status}.")
            }
            Self::RustcUtf8(err) => {
                write!(
                    f,
                    "Query for `target-libdir` and `cfg` returned non-UTF-8 data.\n{err}",
                )
            }
            Self::NoTargetLibdir => {
                write!(f, "Query for `target-libdir` did not return any output.")
            }
            Self::ReadDir(err, path) => {
                write!(
                    f,
                    "Could not read directory entries of `target-libdir` `{}`.\n{err}",
                    path.display(),
                )
            }
            Self::Copy(err, src, dest) => {
                write!(
                    f,
                    "Could not copy file from `{}` to `{}`.\n{err}",
                    src.display(),
                    dest.display(),
                )
            }
            Self::NotFound(path, ext) => {
                write!(
                    f,
                    "Could not find `libstd-*.{ext}` in `{}`.",
                    path.display(),
                )
            }
        }
    }
}
