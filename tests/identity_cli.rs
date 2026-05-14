use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn mantle_help_reports_canonical_identity() {
    Command::cargo_bin("mantle")
        .expect("mantle binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Mantle build system"))
        .stdout(predicate::str::contains("Usage: mantle"))
        .stdout(predicate::str::contains("Usage: crunch").not());
}

#[test]
fn mantle_version_reports_canonical_binary_name() {
    Command::cargo_bin("mantle")
        .expect("mantle binary should build")
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("mantle "))
        .stdout(predicate::str::contains("crunch").not());
}

#[test]
fn legacy_crunch_binary_is_alias_to_mantle_help() {
    Command::cargo_bin("crunch")
        .expect("legacy crunch binary should build")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Mantle build system"))
        .stdout(predicate::str::contains("Usage: mantle"));
}
