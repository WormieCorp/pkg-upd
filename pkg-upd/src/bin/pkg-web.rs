// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]

use std::fmt::Display;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use human_bytes::human_bytes;
use human_panic::setup_panic;
use log::{error, info};
use md5::Md5;
use pkg_upd::{log_data, logging};
use pkg_web::errors::WebError;
use pkg_web::response::ResponseType;
use pkg_web::{LinkElement, WebRequest, WebResponse};
use sha1::Sha1;
use sha2::{Digest, Sha256, Sha512};
use structopt::StructOpt;
use url::Url;
use yansi::Color;

log_data! {"pkg-web"}

#[derive(StructOpt)]
enum ChecksumType {
    Md5,
    Sha1,
    Sha256,
    Sha512,
}

impl FromStr for ChecksumType {
    type Err = &'static str;

    fn from_str(val: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let val: &str = &val.trim().to_lowercase();

        match val {
            "md5" => Ok(ChecksumType::Md5),
            "sha1" => Ok(ChecksumType::Sha1),
            "sha256" | "sha2" => Ok(ChecksumType::Sha256),
            "sha512" => Ok(ChecksumType::Sha512),
            _ => Err("The value is not a supported checksum type"),
        }
    }
}

impl Display for ChecksumType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            ChecksumType::Md5 => f.write_str("MD5"),
            ChecksumType::Sha1 => f.write_str("SHA1"),
            ChecksumType::Sha256 => f.write_str("SHA256"),
            ChecksumType::Sha512 => f.write_str("SHA512"),
        }
    }
}

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

    /// The checksum to compare the downloaded file with
    #[structopt(long)]
    checksum: Option<String>,

    /// The type of the checksum to compare and/or output to console.
    #[structopt(long, default_value = "sha256", possible_values = &["md5", "sha1", "sha256", "sha512"])]
    checksum_type: ChecksumType,

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

/// Allows testing specific urls by either checking which links will be found
/// on an HTML page, or if a file can be downloaded.
#[derive(StructOpt)]
#[structopt(author = "AdmiringWorm <kim.nordmo@gmail.com>", name = "pkg-web")]
struct Arguments {
    #[structopt(subcommand)]
    cmd: Commands,

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

    let request = WebRequest::create();
    match args.cmd {
        Commands::Parse(args) => parse_website_lone(&request, args.url, args.regex),
        Commands::Download(args) => download_file_once(&request, args),
    }
}

fn parse_website_lone(request: &WebRequest, url: Url, regex: Option<String>) {
    match parse_website(request, url, regex) {
        Ok((parent, elements)) => {
            info!(
                "Successfully parsed '{}'",
                Color::Magenta.paint(parent.link)
            );
            info!(
                "Found {} links on the webpage!",
                Color::Cyan.paint(elements.len())
            );

            for link in elements {
                info!(
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
            error!("Unable to parse the requested website!");
            error!("Error message: {}", err);
            std::process::exit(1);
        }
    }
}

fn parse_website(
    request: &WebRequest,
    url: Url,
    regex: Option<String>,
) -> Result<(LinkElement, Vec<LinkElement>), WebError> {
    let response = request.get_html_response(url.as_str())?;

    if let Some(ref regex) = regex {
        response.read(Some(regex))
    } else {
        response.read(None)
    }
}

fn download_file_once(request: &WebRequest, args: DownloadArguments) {
    let temp_dir = if let Some(work_dir) = &args.work_dir {
        work_dir.clone()
    } else {
        std::env::temp_dir()
    };

    if let Err(err) = download_file(request, args, &temp_dir) {
        error!("Unable to download the file. Error: {}", err);
        std::process::exit(1);
    }
}

fn download_file(
    request: &WebRequest,
    args: DownloadArguments,
    work_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let etag = if let Some(ref etag) = args.etag {
        Some(etag.as_str())
    } else {
        None
    };
    let last_modified = if let Some(ref last_modified) = args.last_modified {
        Some(last_modified.as_str())
    } else {
        None
    };
    let response = request.get_binary_response(args.url.as_str(), etag, last_modified)?;

    match response {
        ResponseType::Updated(_) => {
            info!("No update is necessary!");
        }
        ResponseType::New(mut response, _) => {
            response.set_work_dir(work_dir);
            let (etag, last_modified) = get_info(&response);
            let result = response.read(None)?;
            info!("The following information was given by the server:");
            if !etag.is_empty() {
                print_line("ETag", etag.trim_matches('"'));
            } else {
                print_line("ETag", "None");
            }
            if !last_modified.is_empty() {
                print_line("Last Modified", last_modified);
            } else {
                print_line("Last Modified", "None");
            }
            match generate_checksum(&result, &args.checksum_type) {
                Ok(checksum) => {
                    if let Some(original_checksum) = args.checksum {
                        if original_checksum.to_lowercase() == checksum {
                            info!("Originial Checksum matches checksum of downloaded file");
                        } else {
                            error!("Original Checksum do not match checksum of downloaded file!");
                        }
                    }
                    print_line("Checksum", checksum);
                    print_line("Checksum Type", args.checksum_type);
                }
                Err(err) => error!("Unable to generate checksum: {}", err),
            }

            info!(
                "The resulting file is {} long!",
                Color::Cyan.paint(human_bytes(result.metadata()?.len() as f64))
            );

            let _ = std::fs::remove_file(result);
        }
    }
    Ok(())
}

fn generate_checksum(path: &Path, checksum_type: &ChecksumType) -> Result<String, std::io::Error> {
    match checksum_type {
        ChecksumType::Md5 => generate_checksum_from_hasher(Md5::new(), path),
        ChecksumType::Sha1 => generate_checksum_from_hasher(Sha1::new(), path),
        ChecksumType::Sha256 => generate_checksum_from_hasher(Sha256::new(), path),
        ChecksumType::Sha512 => generate_checksum_from_hasher(Sha512::new(), path),
    }
}

fn generate_checksum_from_hasher<T: Digest + Write>(
    mut hasher: T,
    path: &Path,
) -> Result<String, std::io::Error>
where
    <T as sha2::Digest>::OutputSize: std::ops::Add,
    <<T as sha2::Digest>::OutputSize as std::ops::Add>::Output:
        sha2::digest::generic_array::ArrayLength<u8>,
{
    let mut f = std::fs::File::open(path)?;
    std::io::copy(&mut f, &mut hasher)?;
    let result = hasher.finalize();

    Ok(format!("{:x}", result))
}

fn print_line<T: Display, V: Display>(name: T, value: V) {
    let name_style = Color::Magenta.style();
    let value_style = Color::Cyan.style();

    info!(
        "{:>16} : {}",
        name_style.paint(name),
        value_style.paint(value)
    );
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
