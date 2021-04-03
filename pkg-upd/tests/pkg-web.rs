// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn should_parse_with_correct_information_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("pkg-web")?;

    cmd.args(&["parse", "https://github.com/codecov/codecov-exe/releases"])
        .env("NO_COLOR", "true");
    cmd.assert().success().stdout(
        predicate::str::contains(
            "https://github.com/codecov/codecov-exe/tree/1.13.0 (type: Unknown, title: 1.13.0, \
             version: None, text: 1.13.0)",
        )
        .and(predicate::str::contains(
            "https://github.com/codecov/codecov-exe/releases/download/1.13.0/codecov-win7-x86.zip \
             (type: Binary, title: None, version: None, text: codecov-win7-x86.zip",
        )),
    );
    Ok(())
}

#[test]
fn should_parse_with_regex_command() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("pkg-web")?;

    cmd.args(&["parse", "https://chocolatey.org", "--regex", "github"])
        .env("NO_COLOR", "true");

    cmd.assert().success().stdout(
        predicate::str::contains(
            "https://github.com/chocolatey (type: Unknown, title: None, version: None, text: \
             Chocolatey on GitHub)",
        )
        .and(predicate::str::contains("https://chocolatey.org").count(1)), /* We will just get 1
                                                                            * instance, since it
                                                                            * is the url we
                                                                            * parses */
    );

    Ok(())
}

#[test]
fn should_download_file_and_output_message() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("pkg-web")?;

    cmd.args(&[
        "download",
        "https://github.com/codecov/codecov-exe/releases/download/1.11.0/codecov-linux-x64.zip",
    ])
    .env("NO_COLOR", "true");

    cmd.assert().success().stdout(
        predicate::str::contains("The web server responded with status: 200 OK!")
            .and(predicate::str::contains("Downloading 'https://github.com/"))
            .and(predicate::str::contains("Successfully downloaded"))
            .and(predicate::str::contains(
                "ETag : a9da76dd5aa96fcee6de685cc1996075",
            ))
            .and(predicate::str::contains(
                "Last Modified : Wed, 10 Jun 2020 06:14:18 GMT",
            ))
            .and(predicate::str::contains(
                "Checksum : bed13834d3203a1511128d19a9595c53364a0ab9f4d7926e6343c41b48b0f6e5",
            ))
            .and(predicate::str::contains("Checksum Type : SHA256"))
            .and(predicate::str::contains(
                "The resulting file is 15.6 MB long!",
            )),
    );

    Ok(())
}

#[test]
fn should_not_download_up_to_date_file() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("pkg-web")?;

    cmd.args(&[
        "download",
        "https://github.com/chocolatey/ChocolateyGUI/releases/download/0.18.1/ChocolateyGui.Common.0.18.1.nupkg",
        "--etag",
        "f0e303406002b7449f3a92d94761fea6",
        "--last-modified",
        "Mon, 29 Mar 2021 14:28:12 GMT"])
        .env("NO_COLOR", "true");

    cmd.assert().success().stdout(
        predicate::str::contains("The web server responded with status: 304 Not Modified!")
            .and(predicate::str::contains("No update is necessary!")),
    );

    Ok(())
}
