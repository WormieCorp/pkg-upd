// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains information gathered when parsing an html page, or during manual
//! creation.

use std::collections::HashMap;
use std::fmt::Display;

use pkg_version::Versions;
use reqwest::Url;

/// Defines what type (MIME or extension) the current link
/// is for.
///
/// This can be incorrect in cases
/// where the the link is only checked but not the request have been parsed.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum LinkType {
    /// The current link uses an html extension, or have the mime type of
    /// `text/html`.
    Html,
    /// The current link uses a text document extension, or report the mime type
    /// of being `text/html`.
    Text,
    /// The current link uses a css document extension, or the response reports
    /// the mime type of being `text/css`.
    Css,
    /// The current link uses a json document extension, or the response reports
    /// the mime type of being `text/json`.
    Json,
    /// The current link uses one of the following extensions:
    /// - `.exe`
    /// - `.msi`
    /// - `.7z`
    /// - `.zip`
    ///
    /// or the response reports the mime type of being
    /// `application/octet-stream`.
    Binary,
    /// The current link is not a known type, this could be because no file
    /// extension is used, or the request have been sent to the url.
    Unknown,
}

impl Default for LinkType {
    fn default() -> LinkType {
        LinkType::Unknown
    }
}

impl Display for LinkType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Binary => f.write_str("Binary"),
            Self::Css => f.write_str("StyleSheet"),
            Self::Html => f.write_str("HTML"),
            Self::Json => f.write_str("JSON"),
            Self::Text => f.write_str("Text"),
            Self::Unknown => f.write_str("Unknown"),
        }
    }
}

/// Stores information that are know about the current link.
#[derive(Debug, PartialEq)]
pub struct LinkElement {
    /// The full link of this element.
    /// In most cases this is expected to include the domain, and will only be
    /// without one when it has been created manually.
    pub link: Url,
    /// The title of the link, usually gotten from the html attribute `title`.
    pub title: String,
    /// The inner text or html of this link.
    pub text: String,
    /// The version that was parsed pased on any regex that a user specified
    pub version: Option<Versions>,
    /// The type (either by extension, or mime type) that links are for. (*ie:
    /// html, json, text, binary, etc.).
    pub link_type: LinkType,
    /// Any additional attributes specified for the link that are not stored in
    /// any other field.
    pub attributes: HashMap<String, String>,
}

impl LinkElement {
    /// Creates a new edition of the link element, with the specified link url
    /// and the link type.
    pub fn new(url: Url, link_type: LinkType) -> LinkElement {
        LinkElement {
            link: url,
            link_type,
            ..Default::default()
        }
    }

    /// Returns true if the link element type have been set as being a binary
    /// file, in all other cases it will return false.
    pub fn is_binary(&self) -> bool {
        self.link_type == LinkType::Binary
    }
}

impl Default for LinkElement {
    /// Creates a new default link element, with the url set to example.org.
    fn default() -> LinkElement {
        LinkElement {
            link: Url::parse("https://example.org").unwrap(),
            title: Default::default(),
            text: Default::default(),
            version: None,
            link_type: Default::default(),
            attributes: Default::default(),
        }
    }
}
