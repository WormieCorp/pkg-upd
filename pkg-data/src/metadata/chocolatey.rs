// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains all data that can be used that are specific to chocolatey packages.
//! Variables that are common between different packages managers are located in
//! the default package data section.

use std::collections::HashMap;
use std::fmt::Display;

use pkg_version::Versions;
use serde::{Deserialize, Serialize};
use url::Url;

use crate::prelude::Description;

/// Basic structure to hold information regarding a
/// package that are only specific to creating Chocolatey
/// packages.
///
/// ### Examples
///
/// Creating a new data structure with only default empty values.
/// ```
/// use pkg_data::metadata::chocolatey::ChocolateyMetadata;
///
/// let mut data = ChocolateyMetadata::new();
///
/// println!("{:#?}", data);
/// ```
///
/// Creating a new data structure and initialize it with different values.
/// ```
/// use pkg_data::metadata::chocolatey::ChocolateyMetadata;
///
/// let mut data = ChocolateyMetadata::with_authors(&["My-Username"]);
/// data.set_description_str("Some description");
///
/// println!("{:#?}", data);
/// ```
#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[non_exhaustive]
pub struct ChocolateyMetadata {
    /// Wether to force the Chocolatey package to be created using an lowercase
    /// identifier. This is something required to be used on the Chocolatey
    /// Community repository.
    #[serde(default = "crate::defaults::boolean_true")]
    lowercase_id: bool,

    /// The title of the software.
    pub title: Option<String>,

    /// The copyright of the software
    pub copyright: Option<String>,

    /// The version of the Chocolatey package, can be automatically updated and
    /// is not necessary to initially be set.
    #[serde(default = "crate::defaults::empty_version")]
    pub version: Versions,

    /// The authors/developers of the software that the package will be created
    /// for.
    authors: Vec<String>,

    /// The description of the software.
    pub description: Description,

    /// Wether the license of the software requires users to accept the license.
    #[serde(default = "crate::defaults::boolean_true")]
    pub require_license_acceptance: bool,

    /// The url to the documentation of the software.
    pub documentation_url: Option<Url>,

    /// The url to where bugs or features to the software should be reported.
    pub issues_url: Option<Url>,

    #[serde(default)]
    tags: Vec<String>,

    #[serde(default)]
    release_notes: Option<String>,

    #[serde(default)]
    dependencies: HashMap<String, Versions>,
}

impl ChocolateyMetadata {
    /// Helper function to create new empty structure of Chocolatey metadata.
    pub fn new() -> ChocolateyMetadata {
        ChocolateyMetadata {
            lowercase_id: crate::defaults::boolean_true(),
            title: None,
            copyright: None,
            version: crate::defaults::empty_version(),
            authors: vec![],
            description: Description::None,
            require_license_acceptance: true,
            documentation_url: None,
            issues_url: None,
            tags: vec![],
            release_notes: None,
            dependencies: HashMap::new(),
        }
    }

    /// Returns whether lowercase identifiers are forced for this Chocolatey
    /// package.
    pub fn lowercase_id(&self) -> bool {
        self.lowercase_id
    }

    /// Returns the authors/developers of the software that the package is
    /// created for.
    pub fn authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    /// Returns the description of the software the package is created for.
    pub fn description(&self) -> &Description {
        &self.description
    }

    /// Sets the description of the package
    pub fn set_description(&mut self, description: Description) {
        self.description = description;
    }

    pub fn set_description_str(&mut self, description: &str) {
        self.set_description(Description::Text(description.into()));
    }

    pub fn set_title(&mut self, title: &str) {
        if let Some(ref mut self_title) = self.title {
            self_title.clear();
            self_title.push_str(title);
        } else {
            self.title = Some(title.into());
        }
    }

    pub fn set_copyright(&mut self, copyright: &str) {
        if let Some(ref mut self_copyright) = self.copyright {
            self_copyright.clear();
            self_copyright.push_str(copyright);
        } else {
            self.copyright = Some(copyright.into());
        }
    }

    pub fn set_release_notes(&mut self, release_notes: &str) {
        if let Some(ref mut self_release_notes) = self.release_notes {
            self_release_notes.clear();
            self_release_notes.push_str(release_notes);
        } else {
            self.release_notes = Some(release_notes.into());
        }
    }

    pub fn add_dependencies(&mut self, id: &str, version: &str) {
        self.dependencies
            .insert(id.into(), Versions::parse(version).unwrap());
    }

    pub fn set_dependencies(&mut self, dependencies: HashMap<String, Versions>) {
        self.dependencies = dependencies;
    }

    pub fn set_tags<T>(&mut self, tags: &[T]) -> &Self
    where
        T: Display,
    {
        self.tags.clear();

        for tag in tags.iter() {
            self.tags.push(tag.to_string());
        }

        self
    }

    /// Allows initializing and setting the Chocolatey metadata structure with
    /// the specified authors/developers of the software.
    pub fn with_authors<T>(values: &[T]) -> Self
    where
        T: Display,
    {
        if values.is_empty() {
            panic!("Invalid usage: Authors can not be empty!");
        }

        let mut data = Self::new();

        let mut new_authors = Vec::<String>::with_capacity(values.len());

        for val in values.iter() {
            new_authors.push(val.to_string());
        }

        data.authors = new_authors;

        data
    }
}

impl Default for ChocolateyMetadata {
    fn default() -> ChocolateyMetadata {
        ChocolateyMetadata::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_create_with_expected_values() {
        let expected = ChocolateyMetadata {
            lowercase_id: true,
            title: None,
            copyright: None,
            version: crate::defaults::empty_version(),
            authors: vec![],
            description: Description::None,
            require_license_acceptance: true,
            documentation_url: None,
            issues_url: None,
            tags: vec![],
            release_notes: None,
            dependencies: HashMap::new(),
        };

        let actual = ChocolateyMetadata::new();

        assert_eq!(actual, expected);
    }

    #[test]
    fn default_should_create_with_expected_values() {
        let expected = ChocolateyMetadata {
            lowercase_id: true,
            title: None,
            copyright: None,
            version: crate::defaults::empty_version(),
            authors: vec![],
            description: Description::None,
            require_license_acceptance: true,
            documentation_url: None,
            issues_url: None,
            tags: vec![],
            release_notes: None,
            dependencies: HashMap::new(),
        };

        let actual = ChocolateyMetadata::default();

        assert_eq!(actual, expected);
    }

    #[test]
    #[allow(non_snake_case)]
    fn with_authors_should_set_specified_authors_using_String() {
        let authors = [
            String::from("AdmiringWorm"),
            String::from("Chocolatey-Community"),
        ];

        let actual = ChocolateyMetadata::with_authors(&authors);

        assert_eq!(actual.authors(), authors);
    }

    #[test]
    fn with_authors_should_set_specified_authors_using_reference_str() {
        let authors = ["AdmiringWorm", "Chocolatey"];

        let actual = ChocolateyMetadata::with_authors(&authors);

        assert_eq!(actual.authors(), authors);
    }

    #[test]
    #[should_panic(expected = "Invalid usage: Authors can not be empty!")]
    fn with_authors_should_panic_on_empty_vector() {
        let val: Vec<String> = vec![];
        ChocolateyMetadata::with_authors(&val);
    }

    #[test]
    #[should_panic(expected = "Invalid usage: Authors can not be empty!")]
    fn with_authors_should_panic_on_empty_array() {
        let val: [&str; 0] = [];

        ChocolateyMetadata::with_authors(&val);
    }

    #[test]
    fn lowercase_id_should_return_set_values() {
        let mut data = ChocolateyMetadata::new();
        assert_eq!(data.lowercase_id(), true);
        data.lowercase_id = false;

        let actual = data.lowercase_id();

        assert_eq!(actual, false);
    }

    #[test]
    fn description_should_return_set_values() {
        let mut data = ChocolateyMetadata::new();
        assert_eq!(data.description(), &Description::None);
        data.description = Description::Text("Some kind of description".into());

        let actual = data.description();

        assert_eq!(actual, "Some kind of description");
    }

    #[test]
    fn set_description_should_set_expected_value() {
        let mut data = ChocolateyMetadata::new();
        data.set_description_str("My awesome description");

        assert_eq!(data.description(), "My awesome description");
    }
}
