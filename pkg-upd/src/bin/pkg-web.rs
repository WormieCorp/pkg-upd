// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]

use std::path::{Path, PathBuf};

use human_bytes::human_bytes;
use human_panic::setup_panic;
use humanize_url::humanize_url;
use log::{error, info};
use pkg_upd::{log_data, logging};
use pkg_web::response::ResponseType;
use pkg_web::{LinkElement, WebRequest, WebResponse};
use structopt::StructOpt;
use url::Url;
use yansi::Color;

log_data! {"pkg-web"}

#[derive(StructOpt)]
#[structopt(after_help = "EXAMPLES:
    Parsing all urls
      `parse https://github.com/codecove/codecov-exe/releases/latest`Parsing \
                          on matching urls
      `parse https://github.com/codecove/codecov-exe/releases/latest \
                          --regex '.*\\.zip$'`
    Parsing while extracting version
      `parse https://github.com/codecov/codecov-exe/releases/latest \
                          --regex '/(?P<version>[\\d\\.]+)/.*\\.zip$'`")]
struct ParseArguments {
    /// The url to parse use to test parsing of the program.
    url: Url,

    /// The regular expression to use when parsing the specified `url`.
    #[structopt(long, short)]
    regex: Option<String>,
}

#[derive(StructOpt)]
struct DownloadArguments {
    /// The url of the binary file to download.
    url: Url,

    /// The etag that will be matched against the download folder. If matched no
    /// file will be downloaded.
    #[structopt(long)]
    etag: Option<String>,

    /// The last modified date as a string, this is usually the date that
    /// previously was returned from the server.
    #[structopt(long)]
    last_modified: Option<String>,

    /// The work directory that downloads should be downloaded to. [default:
    /// temp dir]
    #[structopt(long, parse(from_os_str))]
    work_dir: Option<PathBuf>,
}

#[derive(StructOpt)]
enum Commands {
    /// Allows testing a single html parse command using the specified url, and
    /// optionally an regex. This will output the links found on the website.
    Parse(ParseArguments),
    /// Allows downloading a single binary file, by default this command will
    /// use `$TEMP` as the work directory and will remove the downloaded file
    /// afterwards.
    Download(DownloadArguments),
}

#[derive(StructOpt)]
#[structopt(author = "AdmiringWorm <kim.nordmo@gmail.com>")]
struct Arguments {
    #[structopt(subcommand)]
    cmd: Option<Commands>,

    #[structopt(flatten)]
    log: LogData,

    #[structopt(long, global = true, env = "NO_COLOR")]
    no_color: bool,
}

fn main() {
    setup_panic!();
    if cfg!(windows) && !yansi::Paint::enable_windows_ascii() {
        yansi::Paint::disable();
    }
    let args = Arguments::from_args();
    if args.no_color {
        yansi::Paint::disable();
    }
    logging::setup_logging(&args.log).expect("Unable to configure logging of the application!");

    if let Some(cmd) = args.cmd {
        let request = WebRequest::create();
        match cmd {
            Commands::Parse(args) => parse_website_lone(&request, args.url, args.regex),
            Commands::Download(args) => {
                let etag = if let Some(ref etag) = args.etag {
                    Some(etag.as_str())
                } else {
                    None
                };
                let last_modified = if let Some(ref last_modified) = args.last_modified {
                    Some(last_modified.as_ref())
                } else {
                    None
                };
                download_file_once(&request, args.url, etag, last_modified, args.work_dir)
            }
        }
    }
}

fn parse_website_lone(request: &WebRequest, url: Url, regex: Option<String>) {
    match parse_website(request, url, regex) {
        Ok((parent, elements)) => {
            println!(
                "Successfully parsed '{}'",
                Color::Magenta.paint(parent.link)
            );
            println!(
                "Found {} links on the webpage!",
                Color::Cyan.paint(elements.len())
            );

            for link in elements {
                println!(
                    "{} (type: {}, title: {}, version: {}, text: {})",
                    Color::Magenta.paint(link.link),
                    Color::Cyan.paint(link.link_type),
                    Color::Cyan.paint(if link.title.is_empty() {
                        "None".into()
                    } else {
                        link.title
                    }),
                    Color::Cyan.paint(if let Some(version) = link.version {
                        format!("{}", version)
                    } else {
                        "None".into()
                    }),
                    Color::Cyan.paint(link.text)
                );
            }
        }
        Err(err) => {
            eprintln!("Unable to parse the requested website!");
            eprintln!("Error message: {}", err);
        }
    }
}

fn parse_website(
    request: &WebRequest,
    url: Url,
    regex: Option<String>,
) -> Result<(LinkElement, Vec<LinkElement>), Box<dyn std::error::Error>> {
    let response = request.get_html_response(url.as_str())?;

    if let Some(ref regex) = regex {
        response.read(Some(regex))
    } else {
        response.read(None)
    }
}

fn download_file_once(
    request: &WebRequest,
    url: Url,
    etag: Option<&str>,
    last_modified: Option<&str>,
    work_dir: Option<PathBuf>,
) {
    let temp_dir = if let Some(work_dir) = work_dir {
        work_dir
    } else {
        std::env::temp_dir()
    };

    match download_file(request, url, &temp_dir, etag, last_modified) {
        Err(err) => {
            eprintln!("Unable to download file from. Error {}", err);
        }
        _ => {}
    }
}

fn download_file(
    request: &WebRequest,
    url: Url,
    work_dir: &Path,
    etag: Option<&str>,
    last_modified: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let response = request.get_binary_response(url.as_str(), etag, last_modified)?;

    match response {
        ResponseType::Updated(status) => {
            info!(
                "The web server responded with status: {}!",
                Color::Cyan.paint(status)
            );
            info!("No update is necessary!");
        }
        ResponseType::New(mut response, status) => {
            info!(
                "The web server responded with status: {}!",
                Color::Cyan.paint(status)
            );
            let file_name = {
                let file_name = response.file_name();
                if file_name.is_none() {
                    error!("We could not parse the file name! Aborting...");
                    std::process::exit(1);
                }
                file_name.unwrap()
            };

            info!(
                "Downloading {} to: {}!",
                humanize_url(url.as_str()).unwrap(),
                work_dir.join(&file_name).display()
            );

            response.set_work_dir(work_dir);
            let (etag, last_modified) = get_info(&response);
            let result = response.read(None)?;
            info!("{} was downloaded!", file_name);
            info!("The following information was given by the server:");
            if !etag.is_empty() {
                info!("  ETag: {}", etag.trim_matches('"'));
            } else {
                info!("  ETag: None");
            }
            if !last_modified.is_empty() {
                info!("  Last Modified: {}", last_modified);
            } else {
                info!("  Last Modified: None");
            }
            info!(
                "The resulting file is {} long!",
                human_bytes(result.metadata()?.len() as f64)
            );

            let _ = std::fs::remove_file(result);
        }
    }
    Ok(())
}

fn get_info<T: WebResponse>(response: &T) -> (String, String) {
    let headers = response.get_headers();
    let mut etag = String::new();
    let mut last_modified = String::new();

    if let Some(etag_val) = headers.get("etag") {
        etag = etag_val.to_string();
    }
    if let Some(modified_val) = headers.get("last-modified") {
        last_modified = modified_val.to_string();
    }

    (etag, last_modified)
}
