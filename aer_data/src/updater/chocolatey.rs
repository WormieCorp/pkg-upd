// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#![cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]

use std::collections::HashMap;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub enum ChocolateyUpdaterType {
    None,
    Installer,
    Archive,
}

impl Default for ChocolateyUpdaterType {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize), serde(untagged))]
pub enum ChocolateyParseUrl {
    UrlWithRegex { url: Url, regex: String },
    Url(Url),
}

#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct ChocolateyUpdaterData {
    #[cfg_attr(feature = "serialize", serde(default))]
    pub embedded: bool,
    #[cfg_attr(feature = "serialize", serde(default, rename = "type"))]
    pub _type: ChocolateyUpdaterType,
    pub parse_url: Option<ChocolateyParseUrl>,

    regexes: HashMap<String, String>,
}

impl ChocolateyUpdaterData {
    pub fn new() -> ChocolateyUpdaterData {
        ChocolateyUpdaterData {
            embedded: false,
            _type: ChocolateyUpdaterType::default(),
            parse_url: None,
            regexes: HashMap::new(),
        }
    }

    pub fn regexes(&self) -> &HashMap<String, String> {
        &self.regexes
    }

    pub fn add_regex(&mut self, name: &str, value: &str) {
        self.regexes.insert(name.into(), value.into());
    }

    pub fn set_regexes(&mut self, values: HashMap<String, String>) {
        self.regexes = values;
    }
}
