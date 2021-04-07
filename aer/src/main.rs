// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]
use std::path::{Path, PathBuf};

use aer::{log_data, logging};
use aer_upd::data::*;
use aer_upd::parsers;
use aer_upd::web::{WebRequest, WebResponse};
#[cfg(feature = "human")]
use human_panic::setup_panic;
use log::{error, info, trace, warn};
use regex::Regex;
use structopt::StructOpt;
use yansi::Paint;

log_data! {}

#[derive(StructOpt)]
#[structopt(author = env!("CARGO_PKG_AUTHORS"))]
struct Arguments {
    /// The files containing the necessary data (metadata+updater data) that
    /// should be used during the run.
    #[structopt(required = true, parse(from_os_str))]
    package_files: Vec<PathBuf>,

    #[structopt(flatten)]
    log: LogData,
}

fn main() {
    #[cfg(feature = "human")]
    setup_panic!();
    if cfg!(windows) && !Paint::enable_windows_ascii() {
        Paint::disable();
    }

    let args = Arguments::from_args();
    logging::setup_logging(&args.log).expect("Unable to configure logging of the application!");

    // TODO: #11 Run updating on several threads
    for file in args.package_files {
        match run_update(&file) {
            Err(err) => error!("An error occurred during update process: '{}'", err),
            _ => {
                todo!()
            }
        }
    }
}

fn run_update(package_file: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Loading package data from '{}'", "yo");

    let data = parsers::read_file(&package_file)?;
    info!(
        "Successfully loaded package data with identifier '{}'!",
        data.metadata().id()
    );

    // TODO: #12 Validate data according to specified rule set, default would be
    // Core

    // TODO: #13 Run any global before hooks

    let request = WebRequest::create();

    if data.updater().has_chocolatey() {
        let choco = data.updater().chocolatey();
        let (_, urls) = match &choco.parse_url {
            Some(chocolatey::ChocolateyParseUrl::Url(url)) => {
                request.get_html_response(url.as_str())?.read(None)?
            }
            Some(chocolatey::ChocolateyParseUrl::UrlWithRegex { url, ref regex }) => {
                info!("Parsing links on '{}' using regex '{}'", url, regex);
                let (parent, urls) = request.get_html_response(url.as_str())?.read(Some(regex))?;
                if !urls.is_empty() {
                    info!("{} links found, using first one to get links!", urls.len());
                    let url = urls.get(0).unwrap();
                    info!("Parsing links on '{}'", url.link);
                    request.get_html_response(url.link.as_str())?.read(None)?
                } else {
                    (parent, urls)
                }
            }
            _ => {
                warn!("No url have been specified to parse!");
                std::process::exit(5);
            }
        };

        let mut aarch32 = None;
        let mut aarch64 = None;
        let mut others = vec![];

        for (key, regex) in choco.regexes() {
            trace!("Filtering {} urls using {}", key, regex);
            let re = Regex::new(&regex)?;
            let mut items = urls.iter().filter_map(|link| {
                let capture = re.captures(link.link.as_str())?;
                let mut new_link = link.clone();

                if let Ok(version) =
                    Versions::parse(capture.name("version").map(|v| v.as_str()).unwrap_or(""))
                {
                    new_link.version = Some(version);
                }

                Some(new_link)
            });
            info!("Parsing urls matching '{}' for {}", regex, key);

            if key.to_lowercase() == "arch32" {
                info!("Taking first match if found!!");
                aarch32 = items.next();
            } else if key.to_lowercase() == "arch64" {
                info!("Taking first match if found!!");
                aarch64 = items.next();
            } else {
                for link in items {
                    others.push(link);
                }
            }
            if let Some(ref aarch32) = aarch32 {
                info!("Arch 32: {}", aarch32.link);
            } else {
                info!("Arch 32: None")
            }
            if let Some(ref aarch64) = aarch64 {
                info!("Arch 64: {}", aarch64.link);
            } else {
                info!("Arch 64: None");
            }
            {
                let others: Vec<&str> = others.iter().map(|o| o.link.as_str()).collect();
                info!("Others: {:?}", others);
            }
        }

        // TODO: #14 Download architecture files
    }

    Ok(())
}
