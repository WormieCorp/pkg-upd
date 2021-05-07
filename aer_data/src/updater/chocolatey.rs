// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains data that can be used when updating chocolatey packages.
//! Only variables specific to chocolatey packages will be located in
//! this file, with the exception of any potential caching.

#![cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]

use std::collections::HashMap;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use url::Url;

/// Defines the type of chocolatey package that should be created.
/// If it is set to [None][ChocolateyUpdaterType::None] a custom updater
/// script is necessary.
///
/// Other supported types are installer and archive packages.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
pub enum ChocolateyUpdaterType {
    /// Defines that the package is a custom package, and do not use any
    /// existing template.
    None,
    /// Uses the installer template to create a package with the necessary files
    /// for installers.
    Installer,
    /// Uses the archive template to create a package with the necessary files
    /// for archives.
    Archive,
}

impl Default for ChocolateyUpdaterType {
    fn default() -> Self {
        Self::None
    }
}

/// Defines the url to use when parsing for executables (and optionally version,
/// and other urls).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize), serde(untagged))]
pub enum ChocolateyParseUrl {
    /// Defines a url with a specified regex to use to aquire the website that
    /// executable files are located at.
    UrlWithRegex {
        /// The url to parse.
        url: Url,
        /// The regex to use to get the actual executable download page.
        regex: String,
    },
    /// The url to parse.
    Url(Url),
}

/// Contains the necessary data that is needed for aquiring details to use in
/// the package. These details include the url to executables, wether the
/// package is embedded, what type of package it is and more.
#[derive(Debug, Default, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct ChocolateyUpdaterData {
    /// Returns wether the package is meant to be an embedded package, or if it
    /// will download binaries from remote locations.
    ///
    /// Will be ignored if the
    /// [updater_type][ChocolateyUpdaterData::updater_type] is set to
    /// [None][ChocolateyUpdaterType::None].
    #[cfg_attr(feature = "serialize", serde(default))]
    pub embedded: bool,

    /// Returns what type this package is.
    #[cfg_attr(feature = "serialize", serde(default, rename = "type"))]
    pub updater_type: ChocolateyUpdaterType,

    /// Returns the url and optionally regex to use when parsing a web page for
    /// binary files.
    pub parse_url: Option<ChocolateyParseUrl>,

    regexes: HashMap<String, String>,
}

impl ChocolateyUpdaterData {
    /// Creates a new instance of the [ChocolateyUpdaterData] structure with
    /// default values.
    pub fn new() -> ChocolateyUpdaterData {
        ChocolateyUpdaterData {
            embedded: false,
            updater_type: ChocolateyUpdaterType::default(),
            parse_url: None,
            regexes: HashMap::new(),
        }
    }

    /// Returns the regexes to use when parsing links.
    ///
    /// If the regex is named `arch32` or `arch64` these will be stored as the
    /// wanted binary files, and at least one of them is required if the
    /// [update_type][ChocolateyUpdaterData::updater_type] is not set to
    /// [None][ChocolateyUpdaterType::None].
    ///
    /// Anly the first matching instance of the types `arch32` and `arch64` will
    /// be used, any other will store all found links.
    pub fn regexes(&self) -> &HashMap<String, String> {
        &self.regexes
    }

    /// Adds a new instance of a regex with the specified name.
    pub fn add_regex<T: AsRef<str>>(&mut self, name: T, value: T) {
        self.regexes
            .insert(name.as_ref().to_string(), value.as_ref().to_string());
    }

    /// Sets the regexes to use when updating a package.
    pub fn set_regexes<T: AsRef<str>, I: IntoIterator<Item = (T, T)>>(&mut self, values: I) {
        self.regexes.clear();

        for (key, val) in values {
            self.add_regex(key, val);
        }
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
        data.set_regexes(&expected);

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
