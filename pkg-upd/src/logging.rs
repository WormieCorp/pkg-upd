// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
use std::path::PathBuf;

use log::{debug, Level, LevelFilter};
use structopt::StructOpt;
use yansi::{Color, Paint, Style};

#[derive(Copy, Clone)]
struct Colors {
    trace: Style,
    debug: Style,
    info: Style,
    warn: Style,
    error: Style,
}

#[derive(StructOpt)]
pub struct LogData {
    /// The path to where verbose logs should be written.
    #[structopt(
        long = "log",
        alias = "log-file",
        env = "PKG_LOG_PATH",
        global = true,
        parse(from_os_str),
        default_value = concat!("./", env!("CARGO_PKG_NAME"), ".log")
    )]
    pub path: PathBuf,

    /// The log level to use when outputting to the console.
    #[structopt(
        short = "-L",
        long = "log-level",
        env = "PKG_LOG_LEVEL",
        global = true,
        default_value = "info",
        possible_values = &["trace", "debug", "info", "error"]
    )]
    pub level: LevelFilter,
}

impl Colors {
    fn from_level(&self, level: &Level) -> &Style {
        match level {
            Level::Trace => &self.trace,
            Level::Debug => &self.debug,
            Level::Warn => &self.warn,
            Level::Error => &self.error,
            _ => &self.info,
        }
    }

    fn paint<T>(&self, level: &Level, value: T) -> Paint<T> {
        let style = self.from_level(level);

        style.paint(value)
    }

    fn paint_level(&self, level: Level) -> Paint<Level> {
        self.paint(&level, level)
    }
}

pub fn setup_logging(log: &LogData) -> Result<(), Box<dyn std::error::Error>> {
    let colors = Colors {
        trace: Style::new(Color::Black),
        debug: Style::new(Color::Fixed(7)),
        info: Style::new(Color::Unset),
        warn: Style::new(Color::Fixed(208)).bold(),
        error: Style::new(Color::Fixed(196)).bold(),
    };
    let html5ever_level = if log.level > log::LevelFilter::Info {
        log::LevelFilter::Info
    } else {
        log.level
    };

    let cli_info = if log.level > log::LevelFilter::Info {
        fern::Dispatch::new().format(move |out, message, record| {
            let level = record.level();
            out.finish(format_args!(
                "[{}]: {}",
                colors.paint_level(level),
                colors.paint(&level, message)
            ));
        })
    } else {
        fern::Dispatch::new().format(move |out, message, record| {
            out.finish(format_args!("{}", colors.paint(&record.level(), message)));
        })
    }
    .filter(move |metadata| metadata.level() >= log::Level::Info)
    .level(log.level)
    .level_for("html5ever", html5ever_level)
    .chain(std::io::stdout());
    let cli_warn = fern::Dispatch::new()
        .format(move |out, message, record| {
            let level = record.level();
            out.finish(format_args!(
                "[{}]: {}",
                colors.paint_level(level),
                colors.paint(&level, message)
            ));
        })
        .filter(move |metadata| metadata.level() <= log::Level::Warn)
        .level(log.level)
        .chain(std::io::stderr());

    let file_log = fern::Dispatch::new()
        .format(move |out, message, record| {
            let enabled = Paint::is_enabled();
            if enabled {
                Paint::disable();
            }

            out.finish(format_args!(
                "[{}] {} T[{:?}] [{}] {}:{}: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.6f %:z"),
                record.level(),
                std::thread::current().name().unwrap_or("<unnamed>"),
                record.module_path().unwrap_or("<unnamed>"),
                record.file().unwrap_or("<unnamed>"),
                record.line().unwrap_or(0),
                message
            ));
            if enabled {
                Paint::enable();
            }
        })
        .level(log::LevelFilter::Trace)
        .level_for("html5ever", log::LevelFilter::Info)
        .chain(fern::log_file(&log.path)?);

    fern::Dispatch::new()
        .chain(cli_info)
        .chain(cli_warn)
        .chain(file_log)
        .apply()?;

    debug!("Finished configuring logging");

    Ok(())
}
