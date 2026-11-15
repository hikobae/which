mod args;

use std::collections::HashSet;
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

macro_rules! die {
    ($($arg:tt)*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }};
}

/// PATHEXT 環境変数を区切り文字 (";") で分割して返す.
fn parse_pathext(pathext: &str) -> Vec<String> {
    pathext
        .split(";")
        .map(|c| c.trim().to_lowercase())
        .filter(|c| !c.is_empty())
        .collect()
}

/// PATH 環境変数を分割して Vec<PathBuf> を返す.
/// 各エントリの前後の空白は除去し(前後に空白を含むディレクトリ名は扱えない)、空のエントリは捨てる.
/// 非 UTF-8 のエントリは空白除去をせず、そのまま保持する.
fn parse_path(path: &OsStr) -> Vec<PathBuf> {
    env::split_paths(path)
        .map(|p| match p.to_str() {
            Some(s) => PathBuf::from(s.trim()),
            None => p,
        })
        .filter(|p| !p.as_os_str().is_empty())
        .collect()
}

fn usage() -> String {
    format!(
        r#"Usage: {} [-h | --help] [-v | --version] filename

Search the current directory and each directory in PATH for filename,
and print the path of every match.

For each directory, filename is tried as-is first, then with each
extension in PATHEXT.

Options:
  -h, --help     Print usage information and exit.
  -v, --version  Print version number and exit."#,
        env!("CARGO_BIN_NAME")
    )
}

fn version() -> String {
    format!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"))
}

fn which(filename: &str, dirs: &[PathBuf], exts: &[String]) -> Vec<PathBuf> {
    let mut program_paths: Vec<PathBuf> = vec![];
    let mut canonicalized_paths: HashSet<PathBuf> = HashSet::new();

    let exts: Vec<&str> = std::iter::once("")
        .chain(exts.iter().map(String::as_str))
        .collect();
    for dir in dirs {
        for ext in &exts {
            let path = dir.join(format!("{filename}{ext}"));
            if !path.is_file() {
                continue;
            }

            let canonicalized_path = fs::canonicalize(&path).unwrap_or_else(|_| path.clone());
            if canonicalized_paths.insert(canonicalized_path) {
                program_paths.push(path);
            }
        }
    }
    program_paths
}

/// 標準出力へ文字列を書き込む.
fn println(s: &str) -> io::Result<()> {
    let mut out = io::stdout().lock();
    writeln!(out, "{}", s).and_then(|()| out.flush())
}

/// 見つかったパスを 1 行ずつ標準出力へ書く.
fn print_paths(paths: &[PathBuf]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    for path in paths {
        writeln!(out, "{}", path.display())?;
    }
    out.flush()
}

/// 出力結果に応じて終了する. BrokenPipe は黙って成功扱い.
fn exit_with(result: io::Result<()>) -> ! {
    match result {
        Ok(()) => std::process::exit(0),
        // `which foo | head -1` などで下流が先に閉じた場合は黙って終了する.
        Err(e) if e.kind() == io::ErrorKind::BrokenPipe => std::process::exit(0),
        Err(e) => die!("Error: Failed to write to stdout: {}", e),
    }
}

fn main() {
    let args = match args::parse_args() {
        Ok(args) => args,
        Err(args::ArgsError::NotUnicode(os_string)) => die!(
            "Error: Non-UTF-8 argument detected: {}",
            os_string.to_string_lossy()
        ),
    };

    if args.has_help_option {
        exit_with(println(&usage()));
    }

    if args.has_version_option {
        exit_with(println(&version()));
    }

    if args.positional.len() != 1 {
        die!("{}", usage());
    }

    let program_name = &args.positional[0];

    let exts = match env::var("PATHEXT") {
        Ok(pathext) => parse_pathext(&pathext),
        Err(env::VarError::NotPresent) => vec![],
        Err(e) => die!("Error: Failed to read PATHEXT environment variable: {}", e),
    };

    let current_dir = match env::current_dir() {
        Ok(path) => path,
        Err(e) => die!("Error: Failed to get current directory: {}", e),
    };

    let paths = match env::var_os("PATH") {
        Some(path) => [vec![current_dir], parse_path(&path)].concat(),
        None => die!("Error: PATH environment variable is not set"),
    };

    let program_paths = which(program_name, &paths, &exts);
    if program_paths.is_empty() {
        std::process::exit(1);
    }

    exit_with(print_paths(&program_paths));
}
