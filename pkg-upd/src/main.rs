// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]

extern crate pkg_upd;

use std::path::PathBuf;

use human_panic::setup_panic;
use log::{error, info};
use pkg_upd::{log_data, logging};
use structopt::StructOpt;

log_data! {env!("CARGO_PKG_NAME")}

#[derive(StructOpt)]
#[structopt(author = "AdmiringWorm <kim.nordmo@gmail.com>")]
struct Arguments {
    /// The files containing the necessary data (metadata+updater data) that
    /// should be used during the run.
    #[structopt(name = "PKG_FILE", required = true, parse(from_os_str))]
    package_files: Vec<PathBuf>,

    #[structopt(flatten)]
    log: LogData,
}

fn main() {
    setup_panic!();
    if cfg!(windows) && !yansi::Paint::enable_windows_ascii() {
        yansi::Paint::disable();
    }

    let arguments = Arguments::from_args();
    logging::setup_logging(&arguments.log)
        .expect("Unable to configure logging of the application!");

    for file in arguments.package_files {
        let data = match pkg_upd::parsers::read_file(&file) {
            Ok(data) => data,
            Err(error) => {
                error!("Error reading package file: {}", error);
                continue;
            }
        };

        info!(
            "Should continue with updating package: {}",
            data.metadata().id()
        );

        // let file = data.updater().chocolatey().unwrap().

        // if let Some(ref file) = file {
        //     let cwd = std::env::current_dir().unwrap();

        //     let data = run_script(&cwd, PathBuf::from_str(file).unwrap(),
        // &mut data);     info!("{:?}", data);
        // }
    }

    info!("Hello, world!");
}
