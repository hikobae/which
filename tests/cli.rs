use assert_cmd::Command;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::PathCreateDir;

#[test]
fn test_success() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(env!("CARGO_PKG_NAME"))
        .assert()
        .success()
        .stdout(predicates::str::contains(env!("CARGO_PKG_NAME")))
        .code(0);
}

#[test]
fn test_help() {
    for arg in ["-h", "--help"] {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg(arg)
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage:"))
            .code(0);
    }
}

#[test]
fn test_no_args() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.assert()
        .failure()
        .stderr(predicates::str::contains("Usage:"))
        .code(1);
}

#[test]
fn test_too_long_args() {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("a")
        .arg("b")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Usage:"))
        .code(1);
}

#[test]
fn test_missing_env_vars() {
    for env in ["PATHEXT", "PATH"] {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg(env!("CARGO_PKG_NAME"))
            .env_remove(env)
            .assert()
            .failure()
            .stderr(predicates::str::contains("Error:"))
            .code(1);
    }
}

#[test]
fn test_error_when_app_name_is_directory_in_path() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let which_dir = temp_dir.child("which");
    which_dir.create_dir_all().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(env!("CARGO_PKG_NAME"))
        .env_remove("PATH")
        .env("PATH", temp_dir.display().to_string())
        .assert()
        .failure()
        .stdout(predicates::str::is_empty())
        .code(1);
}
