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
    pub updater_type: ChocolateyUpdaterType,
    pub parse_url: Option<ChocolateyParseUrl>,

    regexes: HashMap<String, String>,
}

impl ChocolateyUpdaterData {
    pub fn new() -> ChocolateyUpdaterData {
        ChocolateyUpdaterData {
            embedded: false,
            updater_type: ChocolateyUpdaterType::default(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_create_data_with_expected_values() {
        let expected = ChocolateyUpdaterData {
            embedded: false,
            updater_type: ChocolateyUpdaterType::default(),
            parse_url: None,
            regexes: HashMap::new(),
        };

        let actual = ChocolateyUpdaterData::new();

        assert_eq!(actual, expected);
    }

    #[test]
    fn set_regexes_should_set_expected_values() {
        let mut expected = HashMap::new();
        expected.insert("arch32".to_string(), "test-regex-1".to_string());
        expected.insert("arch64".to_string(), "test-regex-2".to_string());

        let mut data = ChocolateyUpdaterData::new();
        data.set_regexes(expected.clone());

        assert_eq!(data.regexes(), &expected);
    }

    #[test]
    fn add_regex_should_include_new_regex() {
        let mut expected = HashMap::new();
        expected.insert("some".to_string(), "test-addition-regex".to_string());

        let mut data = ChocolateyUpdaterData::new();
        data.add_regex("some", "test-addition-regex");

        assert_eq!(data.regexes(), &expected);
    }
}
