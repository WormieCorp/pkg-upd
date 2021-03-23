// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]

extern crate pkg_upd;

use std::path::PathBuf;

use human_panic::setup_panic;
use log::{error, info};
use pkg_upd::logging::setup_logging;
use pkg_upd::rules::{MessageType, RuleKind, RuleMessage};
use structopt::StructOpt;
use yansi::{Color, Style};

/// Validates that the specified meta file is using valid structer, can use the
/// download locations and that the specified metadata conforms to the wanted
/// rules.
#[derive(StructOpt)]
#[structopt(author = "AdmiringWorm <kim.nordmo@gmail.com>")]
struct Arguments {
    /// The path to the meta file that should be validated.
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// The rule that the metadata should confirm to.
    ///
    /// By using the default or explicitly specifying the `core` rule, only
    /// metadata that would prevent the creation of a package would be
    /// validated.
    ///
    /// Specifying `communty` validates all implemented metadata rules against
    /// best practices when pushing to a community repository. Requirements
    /// would be reported as errors and prevent further processing after the
    /// metadata, while Guidelines and suggestions would be reported as
    /// Warnings.
    #[structopt(long = "rule", default_value, env = "PKG_VALIDATE_RULE", possible_values = &["core", "community"])]
    rule: RuleKind,

    #[structopt(flatten)]
    log: pkg_upd::logging::LogData,
}

fn main() {
    setup_panic!();

    run().unwrap(); // We do unwrap here, and rely on human_panic to display any errors to the user in case of failure.
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = Arguments::from_args();
    setup_logging(&arguments.log)?;

    info!("Loading metadata file from '{}'", arguments.file.display());

    let data = match pkg_upd::parsers::read_file(&arguments.file) {
        Ok(data) => {
            info!("Loaded metadata file successfully!");
            data
        }
        Err(err) => {
            error!("Failed to load metadata file. Failure message: \n\t{}", err);
            return Ok(());
        }
    };

    validate_metadata(&data, &arguments.rule);

    Ok(())
}

fn validate_metadata(data: &pkg_data::PackageData, rule_kind: &RuleKind) {
    let metadata = data.metadata();

    if let Err(rules) = pkg_upd::rules::validate_metadata(metadata, rule_kind) {
        info!(
            "{}",
            Style::new(Color::Yellow)
                .paint("The following issues was found during validation of the package data!")
        );

        let types = &[
            MessageType::Requirement,
            MessageType::Guideline,
            MessageType::Suggestion,
            MessageType::Note,
        ];

        for t in types {
            write_rule_messages(*t, rules.iter().filter(|r| &r.message_type == t));
        }
    } else {
        println!(
            "{}",
            Style::new(Color::Green).paint("No issues was found during validation!")
        );
    }
}

fn write_rule_messages<'a>(
    message_type: MessageType,
    rules: impl Iterator<Item = &'a RuleMessage>,
) {
    let mut write_header = true;

    for rule in rules {
        if write_header {
            let (msg, color) = match message_type {
                MessageType::Requirement => ("REQUIREMENTS", Color::Red),
                MessageType::Guideline => ("GUIDELINES", Color::Yellow),
                MessageType::Suggestion => ("SUGGESTIONS", Color::Cyan),
                MessageType::Note => ("NOTES", Color::Magenta),
            };

            println!("\n{}", color.style().bold().paint(msg));
            write_header = false;
        }

        if rule.package_manager.is_empty() {
            println!("- {}", rule.message);
        } else {
            println!("- {}: {}", rule.package_manager, rule.message);
        }
    }
}
