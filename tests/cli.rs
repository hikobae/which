use assert_cmd::Command;
use assert_fs::fixture::FileTouch;
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

/// 別ファイルに誤マッチしないことを確認するテスト。
/// ファイル名にドットが含まれる場合、拡張子の候補にドットが含まれるため、誤マッチする可能性がある。
#[test]
fn test_dot_in_filename_does_not_match_other_file() {
    const FILE_NAME: &str = "file";
    const FILE_NAME_WITH_DOT: &str = "file.dot";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();
    temp_dir.child(FILE_NAME_WITH_DOT).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME_WITH_DOT)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .assert()
        .success()
        .stdout(format!(
            "{}\n",
            temp_dir.child(FILE_NAME_WITH_DOT).display()
        ))
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
fn test_help_with_extra_args() {
    for arg in ["-h", "--help"] {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg(arg)
            .arg(env!("CARGO_PKG_NAME"))
            .assert()
            .success()
            .stdout(predicates::str::contains("Usage:"))
            .code(0);
    }
}

#[test]
fn test_version() {
    for arg in ["-v", "--version"] {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg(arg)
            .assert()
            .success()
            .stdout(predicates::str::contains(env!("CARGO_PKG_VERSION")))
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
