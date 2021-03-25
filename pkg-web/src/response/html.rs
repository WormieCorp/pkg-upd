// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use regex::{Captures, Regex};
use reqwest::blocking::Response;
use reqwest::{header, Url};
use select::document::Document;
use select::predicate::Name;
use semver::Version;

use crate::response::MIME_TYPES;
use crate::{LinkElement, LinkType, WebResponse};

/// Contains functions and structure for holding a single html response, and
/// extracting any necessary information out of the html page.
///
/// Implements the [WebResponse] trait, and are not meant to be created directly
/// by a user.
#[derive(Debug)]
pub struct HtmlResponse {
    response: Response,
}

impl HtmlResponse {
    /// Creates a new instance of the [HtmlResponse] structe to hold the current
    /// response, and allow reading the content from that response.
    pub fn new(response: Response) -> HtmlResponse {
        HtmlResponse { response }
    }
}

impl WebResponse for HtmlResponse {
    /// Sets the response type that will be returned when calling the
    /// [read](HtmlResponse::read) function. The first item is the link the
    /// response came from, and the second item holds a vector of different
    /// link elements that were found on the html page.
    type ResponseContent = (LinkElement, Vec<LinkElement>);

    fn response(&self) -> &Response {
        &self.response
    }

    /// Reads the current response, and extracts any link elements that were
    /// found in the body as well as the link that were used to get the response
    /// itself. This function can return will return an error if the
    /// response do not have a successful status code, or if the reading of the
    /// body fails.
    fn read(self, re: Option<&str>) -> Result<Self::ResponseContent, Box<dyn std::error::Error>> {
        {
            let response = &self.response;
            if !response.status().is_success() {
                let response = self.response;
                response.error_for_status()?;
                unreachable!();
            }
        }
        let response_url = self.response.url().clone();

        let parent_link = get_parent_link_element(&self);

        let body = self.response.text()?;
        let links = get_link_elements(body, response_url, re)?;

        Ok((parent_link, links))
    }
}

fn get_parent_link_element<T: WebResponse>(content: &T) -> LinkElement {
    let headers = content.get_headers();
    let url = content.response().url();
    let response_type = headers
        .get(header::CONTENT_TYPE.as_str())
        .unwrap_or(&"UNKNOWN");

    for (key, val) in MIME_TYPES.iter() {
        if response_type.contains(key) {
            return LinkElement::new(url.clone(), *val);
        }
    }

    LinkElement::new(url.clone(), LinkType::Unknown)
}

fn get_link_elements(
    text: String,
    parent_url: Url,
    re: Option<&str>,
) -> Result<Vec<LinkElement>, Box<dyn std::error::Error>> {
    let document = Document::from(text.as_str());

    let re = if let Some(re) = re {
        Some(Regex::new(&re)?)
    } else {
        None
    };

    let results = document
        .find(Name("a"))
        .filter_map(|n| {
            let mut link = {
                let href = match n.attr("href") {
                    Some(n) => {
                        if n.is_empty() {
                            return None;
                        } else {
                            n
                        }
                    }
                    _ => return None,
                };

                let href =
                    if href.starts_with('/') || href.starts_with('.') || href.starts_with('#') {
                        parent_url.join(&href)
                    } else {
                        Url::parse(href)
                    }
                    .ok()?;
                LinkElement::new(href, LinkType::Unknown)
            };

            if let Some(re) = &re {
                let capture = re.captures(link.link.as_str())?;
                link.version = parse_version(capture);
            }

            link.text = n.text().trim().into();

            for (key, val) in n.attrs() {
                let key = key.to_lowercase();
                if key == "href" {
                    continue;
                } else if key == "title" {
                    link.title = val.into();
                } else {
                    let _ = link.attributes.insert(key, val.into());
                }
            }

            let path = link.link.path();
            if path.ends_with(".html") {
                link.link_type = LinkType::Html;
            } else if path.ends_with(".json") {
                link.link_type = LinkType::Json;
            } else if path.ends_with(".css") {
                link.link_type = LinkType::Css;
            } else if path.ends_with(".txt") {
                link.link_type = LinkType::Text;
            } else if path.ends_with(".zip")
                || path.ends_with(".7z")
                || path.ends_with(".exe")
                || path.ends_with(".msi")
                || path.ends_with(".tar")
                || path.ends_with(".tar.gz")
                || path.ends_with(".tar.bz2")
                || path.ends_with(".nupkg")
            {
                link.link_type = LinkType::Binary;
            }

            Some(link)
        })
        .collect();

    Ok(results)
}

fn parse_version(captures: Captures<'_>) -> Option<Version> {
    lenient_semver::parse(captures.name("version")?.as_str()).ok()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use semver::Version;

    use super::*;
    use crate::WebRequest;

    #[test]
    fn read_should_get_links_from_page() {
        let request = WebRequest::create();
        let url = Url::parse("https://httpbin.org/links/4/1").unwrap();
        let response = request.get_html_response(url.as_ref()).unwrap();

        let (parent, links) = response.read(None).unwrap();

        assert_eq!(parent, LinkElement::new(url, LinkType::Html));
        assert_eq!(
            links,
            [
                LinkElement {
                    link: Url::parse("https://httpbin.org/links/4/0").unwrap(),
                    text: "0".into(),
                    ..Default::default()
                },
                LinkElement {
                    link: Url::parse("https://httpbin.org/links/4/2").unwrap(),
                    text: "2".into(),
                    ..Default::default()
                },
                LinkElement {
                    link: Url::parse("https://httpbin.org/links/4/3").unwrap(),
                    text: "3".into(),
                    ..Default::default()
                },
            ]
        );
    }

    #[test]
    fn read_should_extract_version_from_parsed_links() {
        let request = WebRequest::create();
        let response = request
            .get_html_response("https://github.com/MASGAU/MASGAU/releases/tag/v.1.0.6")
            .unwrap();

        let links = response
            .read(Some(r"/([v\.]+)(?P<version>[\d\.]+)/.*\.exe$"))
            .unwrap()
            .1;

        assert_eq!(links, [
            LinkElement {
                link: Url::parse("https://github.com/MASGAU/MASGAU/releases/download/v.1.0.6/MASGAU-1.0.6-Release-Setup.exe").unwrap(),
                link_type: LinkType::Binary,
                title: "".into(),
                text: "MASGAU v.1.0.6 for Windows".into(),
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("rel".into(), "nofollow".into());
                    map.insert("class".into(), "d-flex flex-items-center min-width-0".into());
                    map
                },
                version: Some(Version::parse("1.0.6").unwrap())
            }
        ])
    }

    #[test]
    fn read_should_only_return_links_matching_specified_regex() {
        let request = WebRequest::create();
        let response = request
            .get_html_response("https://github.com/GitTools/GitReleaseManager/releases/tag/0.11.0")
            .unwrap();

        let links = response.read(Some(r"\.nupkg$")).unwrap().1;

        let expected_items = [
            LinkElement {
                link: Url::parse("https://github.com/GitTools/GitReleaseManager/releases/download/0.11.0/GitReleaseManager.0.11.0.nupkg".into()).unwrap(),
                link_type: LinkType::Binary,
                title: "".into(),
                text: "GitReleaseManager.0.11.0.nupkg".into(),
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("rel".into(), "nofollow".into());
                    map.insert("class".into(), "d-flex flex-items-center min-width-0".into());

                    map
                },
                version: None
            },
            LinkElement {
                link: Url::parse("https://github.com/GitTools/GitReleaseManager/releases/download/0.11.0/gitreleasemanager.portable.0.11.0.nupkg".into()).unwrap(),
                link_type: LinkType::Binary,
                title: "".into(),
                text: "gitreleasemanager.portable.0.11.0.nupkg".into(),
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("rel".into(), "nofollow".into());
                    map.insert("class".into(), "d-flex flex-items-center min-width-0".into());

                    map
                },
                version: None
            },
            LinkElement {
                link: Url::parse("https://github.com/GitTools/GitReleaseManager/releases/download/0.11.0/GitReleaseManager.Tool.0.11.0.nupkg".into()).unwrap(),
                link_type: LinkType::Binary,
                title: "".into(),
                text: "GitReleaseManager.Tool.0.11.0.nupkg".into(),
                attributes: {
                    let mut map = HashMap::new();
                    map.insert("rel".into(), "nofollow".into());
                    map.insert("class".into(), "d-flex flex-items-center min-width-0".into());

                    map
                },
                version: None
            },
        ];

        assert_eq!(links, expected_items)
    }

    #[test]
    #[should_panic(expected = "Status(500)")]
    fn read_should_return_error_on_error_response() {
        let request = WebRequest::create();
        let response = request.get_html_response("https://httpbin.org/status/500");

        if let Ok(response) = response {
            let _ = response.read(None).unwrap();
        }
    }

    #[test]
    fn read_should_return_correct_links() {
        let request = WebRequest::create();
        let response = request
            .get_html_response("https://github.com/codecov/codecov-exe/releases/tag/1.13.0")
            .unwrap();
        let (parent, links) = response.read(None).unwrap();

        assert_eq!(
            parent,
            LinkElement::new(
                Url::parse("https://github.com/codecov/codecov-exe/releases/tag/1.13.0").unwrap(),
                LinkType::Html
            )
        );

        assert_eq!(
            links
                .iter()
                .filter(|l| !l.title.is_empty())
                .collect::<Vec<&LinkElement>>()
                .len(),
            3
        );
        assert_eq!(
            links
                .iter()
                .filter(|l| l.link_type == LinkType::Binary)
                .collect::<Vec<&LinkElement>>()
                .len(),
            6
        );
    }
}
