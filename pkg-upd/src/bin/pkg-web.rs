// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![windows_subsystem = "console"]

use pkg_upd::logging;
use pkg_web::{LinkElement, WebRequest, WebResponse};
use structopt::StructOpt;
use url::Url;
use yansi::Color;

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
struct DownloadArguments {}

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
    log: logging::LogData,
}

fn main() {
    let args = Arguments::from_args();
    logging::setup_logging(&args.log).expect("Unable to configure logging of the application!");

    if let Some(cmd) = args.cmd {
        let request = WebRequest::create();
        match cmd {
            Commands::Parse(args) => parse_website_lone(&request, args.url, args.regex),
            _ => {
                unimplemented!()
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
