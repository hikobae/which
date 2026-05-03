use assert_cmd::Command;

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
