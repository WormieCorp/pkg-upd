// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
use std::fmt::Display;

use human_panic::setup_panic;
use pkg_version::*;
use structopt::StructOpt;
use yansi::Color;

/// Parses version strings and outputs the converted values to the version that
/// will be used by different package managers. Additionally shows the
/// equivalent Semantic Version (as per Rust specifications).
#[derive(StructOpt)]
#[structopt(author = "AdmiringWorm <kim.nordmo@gmail.com>", name = "pkg-ver")]
struct Arguments {
    /// The Versions to test what they would be transformed to (*multiple values
    /// can be specified*).
    #[structopt(required = true, name = "VERSIONS")]
    versions: Vec<String>,
}

fn main() {
    setup_panic!();
    if cfg!(windows) && !yansi::Paint::enable_windows_ascii() {
        yansi::Paint::disable();
    }

    let args = Arguments::from_args();

    println!(
        "Checking {} {}...",
        args.versions.len(),
        if args.versions.len() == 1 {
            "version"
        } else {
            "versions"
        }
    );

    for version in args.versions {
        println!();
        print_line("Raw Version", &version);
        let choco = chocolatey::ChocoVersion::parse(&version);
        if let Ok(choco) = choco {
            print_line("Chocolatey", &choco);
            let semver: SemVersion = choco.into();
            print_line("SemVer from Choco", semver);
        } else {
            print_line("Chocolatey", "None");
            print_line("SemVer from Choco", "None");
        }

        let semver = SemVersion::parse(&version);
        if let Ok(semver) = semver {
            print_line("SemVer", &semver);
            let choco: chocolatey::ChocoVersion = semver.into();
            print_line("Choco from SemVer", choco);
        } else {
            print_line("SemVer", "None");
            print_line("Choco from SemVer", "None");
        }
    }
}

fn print_line<T: Display, V: Display>(name: T, value: V) {
    let name_style = Color::Magenta.style();
    let value_style = Color::Cyan.style();

    println!(
        "{:<18}: {}",
        name_style.paint(name),
        value_style.paint(value)
    );
}
