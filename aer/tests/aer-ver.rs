// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
use std::path::PathBuf;
use std::process::Command;

use assert_cmd::prelude::*;
use lazy_static::lazy_static;
use predicates::prelude::*;

lazy_static! {
    static ref LOG_DIR: PathBuf = std::env::temp_dir();
}

#[test]
fn testing_single_item_with_valid_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aer-ver")?;
    let log_path = LOG_DIR.join("aer-ver-tests-parse-single-valid.log");

    cmd.args(&["4.5.1", "--log", log_path.to_str().unwrap()])
        .env("NO_COLOR", "true");

    cmd.assert().success().stdout(predicate::eq(
        "Checking 1 version...

       Raw Version : 4.5.1

        Chocolatey : 4.5.1
 SemVer from Choco : 4.5.1

            SemVer : 4.5.1
 Choco from SemVer : 4.5.1
",
    ));

    Ok(())
}

#[test]
fn testing_single_item_with_invalid_versions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aer-ver")?;
    let log_path = LOG_DIR.join("aer-ver-tests-parse-single-invalid.log");

    cmd.args(&["invalid-ver", "--log", log_path.to_str().unwrap()])
        .env("NO_COLOR", "true");

    cmd.assert().success().stdout(predicate::eq(
        "Checking 1 version...

       Raw Version : invalid-ver

        Chocolatey : None
 SemVer from Choco : None

            SemVer : None
 Choco from SemVer : None
",
    ));

    Ok(())
}

#[test]
fn testing_multiple_versions() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aer-ver")?;
    let log_path = LOG_DIR.join("aer-ver-tests-multiple.log");

    cmd.args(&["3.2.1", "5.2-alpha.5", "--log", log_path.to_str().unwrap()])
        .env("NO_COLOR", "true");

    cmd.assert().success().stdout(predicate::eq(
        "Checking 2 versions...

       Raw Version : 3.2.1

        Chocolatey : 3.2.1
 SemVer from Choco : 3.2.1

            SemVer : 3.2.1
 Choco from SemVer : 3.2.1

       Raw Version : 5.2-alpha.5

        Chocolatey : 5.2-alpha0005
 SemVer from Choco : 5.2.0-alpha.5

            SemVer : None
 Choco from SemVer : None
",
    ));

    Ok(())
}

#[test]
fn testing_with_trace_logging() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("aer-ver")?;
    let log_path = LOG_DIR.join("aer-ver-trace.log");

    cmd.args(&[
        "4.2.1-alpha54.2",
        "--log-level",
        "trace",
        "--log",
        log_path.to_str().unwrap(),
        "--no-color",
    ]);

    cmd.assert().success().stdout(predicate::eq(
        "[DEBUG]: Finished configuring logging
[INFO]: Checking 1 version...

[INFO]:        Raw Version : 4.2.1-alpha54.2

[INFO]:         Chocolatey : 4.2.1-alpha0054-0002
[INFO]:  SemVer from Choco : 4.2.1-alpha-54.2

[INFO]:             SemVer : 4.2.1-alpha54.2
[INFO]:  Choco from SemVer : 4.2.1-alpha0054-0002
",
    ));

    Ok(())
}
