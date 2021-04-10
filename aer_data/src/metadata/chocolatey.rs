// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains all data that can be used that are specific to chocolatey packages.
//! Variables that are common between different packages managers are located in
//! the default package data section.

#![cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]

use std::collections::HashMap;
use std::fmt::Display;
use std::path::{Path, PathBuf};

use aer_version::Versions;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use url::Url;

use crate::prelude::*;

/// Basic structure to hold information regarding a
/// package that are only specific to creating Chocolatey
/// packages.
///
/// ### Examples
///
/// Creating a new data structure with only default empty values.
/// ```
/// use aer_data::metadata::chocolatey::ChocolateyMetadata;
///
/// let mut data = ChocolateyMetadata::new();
///
/// println!("{:#?}", data);
/// ```
///
/// Creating a new data structure and initialize it with different values.
/// ```
/// use aer_data::metadata::chocolatey::ChocolateyMetadata;
///
/// let mut data = ChocolateyMetadata::with_authors(&["My-Username"]);
/// data.set_description_str("Some description");
///
/// println!("{:#?}", data);
/// ```
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct ChocolateyMetadata {
    /// Wether to force the Chocolatey package to be created using an lowercase
    /// identifier. This should always be true (_the default_) on new packages
    /// that will be pushed to the Chocolatey Community repository.
    #[cfg_attr(
        feature = "serialize",
        serde(default = "crate::defaults::boolean_true")
    )]
    lowercase_id: bool,

    #[cfg_attr(feature = "serialize", serde(default))]
    id: String,

    #[cfg_attr(feature = "serialize", serde(default = "crate::defaults::maintainer"))]
    maintainers: Vec<String>,

    /// The short summary of the application, usually what is used as a
    /// description on other package managers.
    ///
    /// This variable will be re-using the [`summary`] element from the global
    /// [package metadata struct] when creating a chocolatey package, or running
    /// hooks/scripts.
    ///
    /// [`summary`]: crate::metadata::PackageMetadata::summary
    /// [package metadata struct]: crate::metadata::PackageMetadata
    pub summary: Option<String>,

    /// The website that the software this package will be created for is
    /// hosted.
    ///
    /// This variable will be re-using the [`project_url`] element from the
    /// global [package metadata struct] when creating a chocolatey package, or
    /// running hooks/scripts.
    ///
    /// [`project_url`]: crate::metadata::PackageMetadata::project_url
    /// [package metadata struct]: crate::metadata::PackageMetadata
    pub project_url: Option<Url>,

    /// The location were the source code of the software is hosted, or were the
    /// source code can be downloaded (not direct downloads).
    ///
    /// This variable will be re-using the [`project_source_url`] elment from
    /// the global [package metadata struct] when creating a chocolatey package,
    /// or running hooks/scripts.
    ///
    /// [`project_source_url`]: PackageMetadata::project_source_url
    /// [package metadata struct]: PackageMetadata
    pub project_source_url: Option<Url>,

    /// The location were the source of this package is hosted.
    ///
    /// This variable will be re-using the [`package_source_url`] element from
    /// the global [package metadata struct] when creating a chocolatey package,
    /// or running hooks/scripts.
    ///
    /// [`package_source_url`]: PackageMetadata::package_source_url
    /// [package metadata struct]: PackageMetadata
    pub package_source_url: Option<Url>,

    /// The full url to the license of the software. The location should be a
    /// public place where the license can be viewed without the need to
    /// download it.
    ///
    /// This variable will be re-using the [`license`] element from the global
    /// [package metadata struct] if possible, when creating a chocolatey
    /// package, or running hooks/scripts.
    ///
    /// [`license`]: crate::metadata::PackageMetadata::license
    /// [package metadata struct]: crate::metadata::PackageMetadata
    pub license_url: Option<Url>,

    /// The title of the software.
    pub title: Option<String>,

    /// The copyright of the software
    pub copyright: Option<String>,

    /// The version of the Chocolatey package, can be automatically updated and
    /// is not necessary to initially be set.
    #[cfg_attr(
        feature = "serialize",
        serde(default = "crate::defaults::empty_version")
    )]
    pub version: Versions,

    /// The authors/developers of the software that the package will be created
    /// for.
    authors: Vec<String>,

    /// The description of the software.
    pub description: Description,

    /// Wether the license of the software requires users to accept the license.
    #[cfg_attr(
        feature = "serialize",
        serde(default = "crate::defaults::boolean_true")
    )]
    pub require_license_acceptance: bool,

    /// The url to the documentation of the software.
    pub documentation_url: Option<Url>,

    /// The url to where bugs or features to the software should be reported.
    pub issues_url: Option<Url>,

    #[cfg_attr(feature = "serialize", serde(default))]
    tags: Vec<String>,

    /// The release notes for the current version being package of the software.
    /// This can also be a link to a remote location instead of including the
    /// release notes inside the package.
    pub release_notes: Option<String>,

    #[cfg_attr(feature = "serialize", serde(default))]
    dependencies: HashMap<String, Versions>,

    #[cfg_attr(feature = "serialize", serde(default))]
    files: HashMap<PathBuf, String>,
}

impl ChocolateyMetadata {
    /// Helper function to create new empty structure of Chocolatey metadata.
    ///
    /// This will generate purely default values for the structure.
    /// Please see [default][Self::default] for more information.
    pub fn new() -> ChocolateyMetadata {
        ChocolateyMetadata::default()
    }

    /// Returns whether lowercase identifiers are forced for this Chocolatey
    /// package.
    ///
    /// Controls the transforming of the [`id`] when it is imported from the
    /// global [package metadata struct]
    ///
    /// [`id`]: PackageMetadata::id
    /// [package metadata struct]: PackageMetadata
    pub fn lowercase_id(&self) -> bool {
        self.lowercase_id
    }

    /// The identifier of the package that will be created.
    /// The identifier should always be in lowercase characters for new
    /// packages.
    ///
    /// This identifier will return an empty string if it has not been
    /// explicitly set, except when creating the chocolatey package or running
    /// any hooks/scripts. In this case it will re-use the [`id`] provided by
    /// the global [package metadata struct] and converted based on valid values
    /// for Chocolatey packages (and respects the [`lowercase_id`] element).
    ///
    /// [`id`]: PackageMetadata::id
    /// [`lowercase_id`]: Self::lowercase_id
    /// [package metadata struct]: PackageMetadata
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the authors/developers of the software that the package is
    /// created for.
    pub fn authors(&self) -> &[String] {
        self.authors.as_slice()
    }

    /// Returns the maintainers of the package that will be created for
    /// chocolatey (owners in nuspec file).
    ///
    /// This will by default be set to the current operating system user, and
    /// will re-use the [`maintainers`] element from the global [package
    /// metadata struct] if it is not changed.
    ///
    /// [`maintainers`]: PackageMetadata::maintainers
    /// [package metadata struct]: PackageMetadata
    pub fn maintainers(&self) -> &[String] {
        self.maintainers.as_slice()
    }

    /// The tags of the current package.
    ///
    /// The first item of the tags will always be equivalent to the lowercase
    /// identifier of the package when it is created, or when running
    /// hooks/scripts.
    pub fn tags(&self) -> &[String] {
        self.tags.as_slice()
    }

    /// The dependencies that should be added to the package, and ensured to be
    /// installed when chocolatey installs the package. This is required by
    /// us to both specify the identifier of the dependency, and a minimum
    /// version.
    pub fn dependencies(&self) -> &HashMap<String, Versions> {
        &self.dependencies
    }

    /// Returns the description of the software the package is created for.
    pub fn description(&self) -> &Description {
        &self.description
    }

    /// The files that should be included with the package, this is similar to
    /// how chocolatey uses `<file src="path" target="destination" /> in the
    /// nuspec. The handling of directory seperators is handled
    /// automatically for the source location.
    ///
    /// By default, only a local path of `tools/**` is set and will always be
    /// included even if custom locations is being used.
    pub fn files(&self) -> &HashMap<PathBuf, String> {
        &self.files
    }

    /// Sets the description of the package
    pub fn set_description(&mut self, description: Description) {
        self.description = description;
    }

    /// Convenience helper for setting a description as a string.
    /// Will most likely be removed and replaced by a way to set it the same way
    /// through the normal function.
    pub fn set_description_str<D: AsRef<str>>(&mut self, description: D) {
        self.set_description(Description::Text(description.as_ref().into()));
    }

    /// Sets the title of the package, the title that will be displayed to
    /// users.
    pub fn set_title<T: AsRef<str>>(&mut self, title: T) {
        if let Some(ref mut self_title) = self.title {
            self_title.clear();
            self_title.push_str(title.as_ref());
        } else {
            self.title = Some(title.as_ref().into());
        }
    }

    /// Sets the copyright of the software that should be included in the
    /// package.
    pub fn set_copyright<C: AsRef<str>>(&mut self, copyright: C) {
        if let Some(ref mut self_copyright) = self.copyright {
            self_copyright.clear();
            self_copyright.push_str(copyright.as_ref());
        } else {
            self.copyright = Some(copyright.as_ref().into());
        }
    }

    /// Sets the release notes of the software, or a url to the location of the
    /// release notes.
    pub fn set_release_notes<R: AsRef<str>>(&mut self, release_notes: R) {
        if let Some(ref mut self_release_notes) = self.release_notes {
            self_release_notes.clear();
            self_release_notes.push_str(release_notes.as_ref());
        } else {
            self.release_notes = Some(release_notes.as_ref().into());
        }
    }

    /// Adds a new dependency to the package, with the specified identifier and
    /// minimum version.
    pub fn add_dependencies<I: AsRef<str>, V: AsRef<str>>(&mut self, id: I, version: V) {
        // TODO: Change version.as_ref() to version when dependency is updated
        self.dependencies.insert(
            id.as_ref().into(),
            Versions::parse(version.as_ref()).unwrap(),
        );
    }

    /// Adds a new file to the package (or globbing pattern), and sets the
    /// specified target destination.
    pub fn add_file<P: AsRef<Path>, T: AsRef<str>>(&mut self, src: P, target: T) {
        self.files
            .insert(PathBuf::from(src.as_ref()), String::from(target.as_ref()));
    }

    /// Adds a new tag to the package.
    pub fn add_tag<T: AsRef<str>>(&mut self, tag: T) {
        self.tags.push(String::from(tag.as_ref()));
    }

    /// Clears and sets the specified dependencies to the package.
    pub fn set_dependencies<K: AsRef<str>, V: AsRef<str>>(&mut self, dependencies: &[(K, V)]) {
        self.dependencies.clear();
        for (key, val) in dependencies {
            self.add_dependencies(key, val);
        }
    }

    /// Clears and sets the specified files for the package.
    pub fn set_files<P: AsRef<Path>, T: AsRef<str>>(&mut self, files: &[(P, T)]) {
        self.files.clear();

        for (src, target) in files {
            self.add_file(src, target);
        }
    }

    /// Clears and sets the specified tags for the package.
    pub fn set_tags<T: AsRef<str>>(&mut self, tags: &[T]) -> &Self {
        self.tags.clear();

        for tag in tags.iter() {
            self.add_tag(tag);
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
    /// Generates a new default instance of the [ChocolateyMetadata] structure,
    /// using default values with the following exceptions.
    ///
    /// - [lowercase_id][Self::lowercase_id] is set by default to `true`
    /// - [maintainers][Self::maintainers] is set by default to the current
    ///   operating system user
    /// - [version][Self::version] is set to a version equivalent to `0.0.0`
    /// - [require_license_acceptance][Self::require_license_acceptance] is set
    ///   by default to `true`
    fn default() -> ChocolateyMetadata {
        ChocolateyMetadata {
            lowercase_id: true,
            id: Default::default(),
            maintainers: crate::defaults::maintainer(),
            summary: None,
            project_url: None,
            project_source_url: None,
            package_source_url: None,
            license_url: None,
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
            files: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn new_should_create_with_expected_values() {
        let expected = ChocolateyMetadata::default();

        let actual = ChocolateyMetadata::new();

        assert_eq!(actual, expected);
    }

    #[test]
    fn default_should_create_with_expected_values() {
        let expected = ChocolateyMetadata {
            lowercase_id: true,
            id: String::new(),
            maintainers: crate::defaults::maintainer(),
            summary: None,
            project_url: None,
            project_source_url: None,
            package_source_url: None,
            license_url: None,
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
            files: HashMap::new(),
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
