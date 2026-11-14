mod args;

use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

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

/// PATH 環境変数を分割して Vec<String> を返す.
fn parse_path(path: &str) -> Vec<String> {
    env::split_paths(path)
        .map(|p| p.to_string_lossy().trim().to_string())
        .filter(|p| !p.is_empty())
        .collect()
}

fn usage() -> String {
    format!(
        r#"Usage: {} filename

which print the pathname of the filename in the current environment.

Options:
-h, --help
  Print usage information on standard output then exit successfully.

-v, --version
  Print version number and quit."#,
        env!("CARGO_BIN_NAME")
    )
}

fn version() -> String {
    format!("{} {}", env!("CARGO_BIN_NAME"), env!("CARGO_PKG_VERSION"))
}

fn which(filename: &str, dirs: &[String], exts: &[String]) -> Vec<PathBuf> {
    let mut program_paths: Vec<PathBuf> = vec![];
    let mut canonicalized_paths: HashSet<PathBuf> = HashSet::new();

    let exts = [vec!["".to_string()], exts.to_vec()].concat();
    for dir in dirs {
        let dir_path = Path::new(dir);
        for ext in &exts {
            let path = dir_path.join(filename.to_string() + ext);
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

fn main() {
    let args = args::parse_args();

    if args.has_help_option {
        println!("{}", usage());
        std::process::exit(0);
    }

    if args.has_version_option {
        println!("{}", version());
        std::process::exit(0);
    }

    if args.positional.len() != 1 {
        die!("{}", usage());
    }

    let program_name = &args.positional[0];

    let exts = match env::var("PATHEXT") {
        Ok(pathext) => parse_pathext(&pathext),
        Err(e) => die!("Error: Failed to read PATHEXT environment variable: {}", e),
    };

    let current_dir = match env::current_dir() {
        Ok(path) => path.display().to_string(),
        Err(e) => die!("Error: Failed to get current directory: {}", e),
    };

    let paths = match env::var("PATH") {
        Ok(path) => [vec![current_dir], parse_path(&path)].concat(),
        Err(e) => die!("Error: Failed to read PATH environment variable: {}", e),
    };

    let program_paths = which(program_name, &paths, &exts);
    if program_paths.is_empty() {
        std::process::exit(1);
    }
    for path in program_paths {
        println!("{}", path.display());
    }
}
