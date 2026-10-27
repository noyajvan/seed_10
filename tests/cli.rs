use std::process::Command;

/// Перевіряємо реальний бінарник, а не тільки бібліотеку.
fn bin() -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_seed_10"));
    cmd.env("NO_COLOR", "1");
    cmd
}

#[test]
fn greets_world_by_default() {
    let out = bin().output().expect("run seed_10");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "Hello, world!");
}

#[test]
fn greets_positional_name() {
    let out = bin().arg("Ada").output().expect("run seed_10 Ada");
    assert!(out.status.success());
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "Hello, Ada!");
}

#[test]
fn help_exits_zero() {
    let out = bin().arg("--help").output().expect("run seed_10 --help");
    assert!(out.status.success());
    assert!(String::from_utf8_lossy(&out.stdout).contains("usage: seed_10"));
}

#[test]
fn bad_flag_exits_two() {
    let out = bin().arg("--nope").output().expect("run seed_10 --nope");
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("usage error"));
}
