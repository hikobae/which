use assert_cmd::Command;
use assert_fs::fixture::FileTouch;
use assert_fs::fixture::PathChild;
use assert_fs::prelude::PathCreateDir;
use std::env;

/// カレントディレクトリ経由で見つかったパスの期待値を返す.
///
/// - unix: 子プロセスの `env::current_dir()` はシンボリックリンク解決済みのパスを返す
///   (macOS の `/var` -> `/private/var` など)。そのため `canonicalize()` で表記を合わせる。
/// - それ以外 (Windows): `canonicalize()` は `\\?\` 付きのパスを返し、`current_dir()` の
///   表記と一致しない。`dir.path()` をそのまま使う。
fn via_cwd(dir: &assert_fs::TempDir, name: &str) -> String {
    #[cfg(unix)]
    let base = dir.path().canonicalize().unwrap();
    #[cfg(not(unix))]
    let base = dir.path().to_path_buf();
    base.join(name).display().to_string()
}

/// PATH 上のファイルが見つかる基本ケース.
#[test]
fn test_success() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// 複数のディレクトリにファイルがある場合、PATH の順に全て出力される.
#[test]
fn test_path_order_with_multiple_matches() {
    const FILE_NAME: &str = "file";
    let temp_dir1 = assert_fs::TempDir::new().unwrap();
    let temp_dir2 = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir1.child(FILE_NAME).touch().unwrap();
    temp_dir2.child(FILE_NAME).touch().unwrap();

    // PATH の指定順を逆にして、ソートされず指定順が保たれることも確認する.
    for (first, second) in [(&temp_dir1, &temp_dir2), (&temp_dir2, &temp_dir1)] {
        let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
        cmd.arg(FILE_NAME)
            .env(
                "PATH",
                env::join_paths([first.path(), second.path()]).unwrap(),
            )
            .env("PATHEXT", ".COM;.EXE")
            .current_dir(&other_dir)
            .assert()
            .success()
            .stdout(format!(
                "{}\n{}\n",
                first.child(FILE_NAME).display(),
                second.child(FILE_NAME).display()
            ))
            .code(0);
    }
}

/// カレントディレクトリのファイルは PATH 上のファイルより先に出力される.
#[test]
fn test_current_directory_comes_first() {
    const FILE_NAME: &str = "file";
    let current_dir = assert_fs::TempDir::new().unwrap();
    let path_dir = assert_fs::TempDir::new().unwrap();
    current_dir.child(FILE_NAME).touch().unwrap();
    path_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", path_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&current_dir)
        .assert()
        .success()
        .stdout(format!(
            "{}\n{}\n",
            via_cwd(&current_dir, FILE_NAME),
            path_dir.child(FILE_NAME).display()
        ))
        .code(0);
}

/// 別ファイルに誤マッチしないことを確認するテスト。
/// ファイル名にドットが含まれる場合、拡張子の候補にドットが含まれるため、誤マッチする可能性がある。
#[test]
fn test_dot_in_filename_does_not_match_other_file() {
    const FILE_NAME: &str = "file";
    const FILE_NAME_WITH_DOT: &str = "file.dot";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();
    temp_dir.child(FILE_NAME_WITH_DOT).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME_WITH_DOT)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
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
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ";.COM;.EXE;")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATH に同じディレクトリが複数回含まれ、カレントディレクトリとも重複していても 1 回だけ出力する.
#[test]
fn test_path_with_duplicate_directories() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let path = temp_dir.display().to_string();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", env::join_paths([&path, &path]).unwrap())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", via_cwd(&temp_dir, FILE_NAME)))
        .code(0);
}

/// PATH に無くても、カレントディレクトリのファイルは見つかる.
#[test]
fn test_current_directory_without_path() {
    const FILE_NAME: &str = "file";
    let current_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    current_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", other_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&current_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", via_cwd(&current_dir, FILE_NAME)))
        .code(0);
}

/// PATHEXT による拡張子補完のテスト
#[test]
fn test_pathext_completion() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child("file.exe").touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg("file")
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child("file.exe").display()))
        .code(0);
}

/// コマンドの引数にフルパスを指定した場合、PATH に無い場所のファイルでも見つかる.
#[test]
fn test_full_path() {
    const FILE_NAME: &str = "file";
    let file_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    let file = file_dir.child(FILE_NAME);
    file.touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(file.path())
        .env("PATH", other_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", file.display()))
        .code(0);
}

/// フルパス(拡張子なし)指定でも PATHEXT による補完が効く.
#[test]
fn test_full_path_with_pathext() {
    let file_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    let file = file_dir.child("file.exe");
    file.touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(file_dir.child("file").path())
        .env("PATH", other_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", file.display()))
        .code(0);
}

/// PATH に大文字小文字だけが異なる同一ディレクトリがあり、カレントディレクトリとも重複していても 1 回だけ出力する.
/// 大文字小文字を区別しないファイルシステム(Windows)でのみ成立する.
#[test]
#[cfg(windows)]
fn test_path_with_case_variant_directories() {
    use std::path::PathBuf;

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
        .current_dir(&temp_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", via_cwd(&temp_dir, FILE_NAME)))
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
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env_remove("PATHEXT")
        .current_dir(&other_dir)
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
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stderr(predicates::str::contains("Error:"))
        .code(1);
}

/// 存在しないアプリ名を指定して、stdout が空・終了コード 1 になることを確認するテスト。
#[test]
fn test_not_found() {
    const FILE_NAME: &str = "file.exe";
    let temp_dir = assert_fs::TempDir::new().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(predicates::str::is_empty())
        .code(1);
}

/// PATH 上の同名のエントリがディレクトリの場合は実行ファイルとして扱わず、
/// 何も出力せず終了コード 1 で終了することを確認するテスト。
#[test]
fn test_directory_with_same_name_is_ignored() {
    const FILE_NAME: &str = "which";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).create_dir_all().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&temp_dir)
        .assert()
        .failure()
        .stdout(predicates::str::is_empty())
        .code(1);
}

/// `-` で始まる未知のオプションはファイル名として扱われる.
#[test]
fn test_dash_prefixed_name_is_treated_as_filename() {
    const FILE_NAME: &str = "-x";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", temp_dir.display().to_string())
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATH に非 UTF-8 のディレクトリが含まれていても、他のディレクトリの検索は動作する.
/// 非 UTF-8 のパスを環境変数に渡せるのは unix のみ.
#[test]
#[cfg(unix)]
fn test_non_unicode_path_entry() {
    use std::os::unix::ffi::OsStringExt;
    use std::path::PathBuf;

    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let invalid = PathBuf::from(std::ffi::OsString::from_vec(vec![b'/', 0xff, 0xfe]));
    let path = env::join_paths([invalid, temp_dir.path().to_path_buf()]).unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", path)
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}

/// PATH の各エントリの前後の空白は除去される.
#[test]
fn test_path_entry_with_surrounding_spaces() {
    const FILE_NAME: &str = "file";
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let other_dir = assert_fs::TempDir::new().unwrap();
    temp_dir.child(FILE_NAME).touch().unwrap();

    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.arg(FILE_NAME)
        .env("PATH", format!("  {}  ", temp_dir.display()))
        .env("PATHEXT", ".COM;.EXE")
        .current_dir(&other_dir)
        .assert()
        .success()
        .stdout(format!("{}\n", temp_dir.child(FILE_NAME).display()))
        .code(0);
}
