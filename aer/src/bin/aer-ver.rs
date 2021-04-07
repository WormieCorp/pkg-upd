// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]
use std::fmt::Display;

use aer::{log_data, logging};
use aer_upd::data::chocolatey::ChocoVersion;
use aer_upd::data::{FixVersion, SemVersion};
#[cfg(feature = "human")]
use human_panic::setup_panic;
use lazy_static::lazy_static;
use log::{error, info};
use structopt::StructOpt;
use yansi::{Color, Paint, Style};

log_data! {"aer-ver"}

/// Parses version strings and outputs the converted values to the version that
/// will be used by the supported package managers. Additionally shows the
/// equivalent Semantic Version (as per Rust specifications) when possible.
#[derive(StructOpt)]
#[structopt(author = env!("CARGO_PKG_AUTHORS"), name = "aer-ver")]
struct Arguments {
    /// The Versions to test what they would be transformed to (*multiple values
    /// can be specified*).
    #[structopt(required = true)]
    versions: Vec<String>,

    #[structopt(flatten)]
    log: LogData,

    /// Disable the usage of colors when outputting text to the console.
    #[structopt(long, global = true)]
    no_color: bool,

    /// Also display what fix version would be created (if the type allows fix
    /// versions).
    #[structopt(long)]
    with_fix_version: bool,
}

fn main() {
    #[cfg(feature = "human")]
    setup_panic!();
    let args = {
        let mut args = Arguments::from_args();
        if std::env::var("NO_COLOR").unwrap_or_default().to_lowercase() == "true" {
            args.no_color = true;
        }

        if args.no_color || (cfg!(windows) && !Paint::enable_windows_ascii()) {
            Paint::disable();
        }
        args
    };

    logging::setup_logging(&args.log).expect("Unable to configure logging of the application!");

    info!(
        "Checking {} {}...",
        args.versions.len(),
        if args.versions.len() == 1 {
            "version"
        } else {
            "versions"
        }
    );

    for version in args.versions {
        println!(); // We don't need to add an empty line in the log file
        print_line("Raw Version", &version);
        println!();
        if let Ok(mut choco) = ChocoVersion::parse(&version) {
            print_line("Chocolatey", &choco);
            let semver: SemVersion = choco.clone().into();
            print_line("SemVer from Choco", semver);
            if args.with_fix_version {
                match choco.add_fix() {
                    Ok(_) => print_line("Chocolatey Fix", &choco),
                    Err(err) => error!("An error occurred while creating fix version: {}", err),
                }
            }
        } else {
            print_line("Chocolatey", "None");
            print_line("SemVer from Choco", "None");
        }
        println!();

        let semver = SemVersion::parse(&version);
        if let Ok(semver) = semver {
            print_line("SemVer", &semver);
            let mut choco: ChocoVersion = semver.into();
            print_line("Choco from SemVer", &choco);
            if args.with_fix_version {
                match choco.add_fix() {
                    Ok(_) => print_line("Chocolatey Fix", &choco),
                    Err(err) => error!("An error occurred while creating fix version: {}", err),
                }
            }
        } else {
            print_line("SemVer", "None");
            print_line("Choco from SemVer", "None");
        }
    }
}

fn print_line<T: Display, V: Display>(name: T, value: V) {
    lazy_static! {
        static ref NAME_STYLE: Style = Color::Magenta.style();
        static ref VALUE_STYLE: Style = Color::Cyan.style();
    };

    info!(
        "{:>18} : {}",
        NAME_STYLE.paint(name),
        VALUE_STYLE.paint(value)
    );
}
