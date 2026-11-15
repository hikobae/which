use assert_cmd::Command;
use assert_fs::fixture::FileTouch;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::PathCreateDir;
use std::env;
use std::path::PathBuf;

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

/// PATHEXTの先頭と末尾にセミコロンがある場合のテスト
#[test]
fn test_pathext_with_leading_trailing_semicolons() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ";.COM;.EXE;")
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATH に同じディレクトリが複数回含まれる場合のテスト
#[test]
fn test_path_with_duplicate_directories() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let path = temp_dir.display().to_string();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", env::join_paths(&[path.clone(), path]).unwrap())
        .env("PATHEXT", ".COM;.EXE")
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATH にカレントディレクトリが含まれる場合のテスト
#[test]
fn test_path_with_current_directory() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// コマンドの引数にフルパスを指定した場合のテスト
#[test]
fn test_full_path() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let file = temp_dir.child(FILE_NAME);
    file.touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(file.path())
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", file.display()))
        .code(0);
}

/// PATH に大文字小文字だけが異なる同一ディレクトリがあっても 1 回だけ出力する.
#[test]
fn test_path_with_case_variant_directories() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let orig = temp_dir.path().to_path_buf();
    let upper = PathBuf::from(orig.to_string_lossy().to_uppercase());
    assert_ne!(orig, upper);

    let path = env::join_paths([&orig, &upper]).unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", path)
        .env("PATHEXT", ".COM;.EXE")
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
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
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(env!("CARGO_PKG_NAME"))
        .env_remove("PATH")
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error:"))
        .code(1);
}

/// PATHEXT 未定義の場合も正常に動作することを確認するテスト
#[test]
fn test_missing_pathext() {
    const FILE_NAME: &str = "file.exe";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env_remove("PATHEXT")
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATHEXT が非 UTF-8 の場合はエラー終了することを確認するテスト
#[test]
fn test_non_unicode_pathext() {
    #[cfg(unix)]
    let invalid = {
        use std::os::unix::ffi::OsStringExt;
        std::ffi::OsString::from_vec(vec![0xff, 0xfe])
    };
    #[cfg(windows)]
    let invalid = {
        use std::os::windows::ffi::OsStringExt;
        std::ffi::OsString::from_wide(&[0xD800])
    };

    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", &invalid)
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error:"))
        .code(1);
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
