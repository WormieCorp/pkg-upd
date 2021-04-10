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

fn generate_identifier<T: AsRef<str>>(id: T, lowercase: bool) -> String {
    if lowercase {
        id.as_ref().to_lowercase()
    } else {
        id.as_ref().into()
    }
    .replace(' ', "-")
}

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

    #[cfg_attr(feature = "serialize", serde(default))]
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

    /// Creates a new instance with the specified identifier.
    /// The identifier is expected to be a valid chocolatey id.
    pub fn with_id<T: AsRef<str>>(id: T, lowercase: bool) -> Self {
        let mut choco = ChocolateyMetadata {
            lowercase_id: lowercase,
            id: generate_identifier(id.as_ref(), lowercase),
            ..Default::default()
        };
        choco.add_tag(generate_identifier(id, true));

        choco
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
            maintainers: Vec::new(),
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

impl DataUpdater<PackageMetadata> for ChocolateyMetadata {
    /// This task updates the [ChocolateyMetadata] with the necessary values
    /// specified in the [PackageMetadata] structure.
    ///
    /// Among the tasks that are being done is as follows:
    ///
    /// - Updates the [id][Self::id] if it is an empty string with the value
    ///   specified in [PackageMetadata::id] while replacing spaces with dashes,
    ///   and makes it lowercase if [lowercase_id][Self::lowercase_id] is set to
    ///   `true`.
    /// - Update the [maintainers][Self::id] if it is empty, or is equal to the
    ///   operating system user with the values from
    ///   [PackageMetadata::maintainers].
    /// - Update the [summary][Self::summary] if it is empty with the values
    ///   from [PackageMetadata::summary].
    /// - Update the [project_url][Self::project_url] if it is empty with the
    ///   values from [PackageMetadata::project_url].
    /// - Update the [project_source_url][Self::project_source_url] if it is
    ///   empty with the values from [PackageMetadata::project_source_url].
    /// - Update the [package_source_url][Self::package_source_url] if it is
    ///   empty with the values from [PackageMetadata::package_source_url].
    /// - Update the [license_url][Self::license_url] if it is empty with the
    ///   values from [PackageMetadata]::license_url. This will only be
    ///   automatically set if the global metadata also have a url specified, or
    ///   if the expression is a known/supported SPDIX.
    /// - Add the recommended identifier (replacing spaces with dashes, and in
    ///   lowercase) as the first item in the tags vector.
    fn update_from<R: AsRef<PackageMetadata>>(&mut self, from: R) {
        let from = from.as_ref();
        if self.id.is_empty() && !from.id().is_empty() {
            self.id = generate_identifier(from.id(), self.lowercase_id());
        }

        if self.maintainers.is_empty() {
            self.maintainers = from.maintainers().iter().map(|m| m.to_string()).collect();
        }

        if self.summary.is_none() && !from.summary.is_empty() {
            self.summary = Some(from.summary.clone());
        }

        if self.project_url.is_none() {
            self.project_url = Some(from.project_url().clone());
        }

        if self.project_source_url.is_none() {
            self.project_source_url = from.project_source_url.clone();
        }

        if self.package_source_url.is_none() {
            self.package_source_url = from.package_source_url.clone();
        }

        if self.license_url.is_none() {
            if let Some(license) = from.license().license_url() {
                self.license_url = Some(Url::parse(license).unwrap());
            }
        }

        let lower_id = self.id().to_lowercase();
        if !self.id().is_empty() && !self.tags.contains(&lower_id) {
            self.tags.insert(0, self.id().to_lowercase());
        }
    }

    /// Reset the variables that are the same, automatically set or not needed.
    ///
    /// Among the tasks that are being done is as follows:
    ///
    /// - Removes the [id][Self::id] if it matches the expected identifier that
    ///   would automatically be created (_depends on the
    ///   [lowercase_id][Self::lowercase_id] variable_).
    /// - Remove the [maintainers][Self::maintainers] if they are the same as
    ///   the global [PackageMetadata::maintainers] variable.
    /// - Remove the [summary][Self::summary] if it is the same as the global
    ///   [PackageMetadata::summary] variable.
    /// - Remove the [project_url][Self::project_url] if it is the same as the
    ///   global [PackageMetadata::project_url] variable.
    /// - Remove the [project_source_url][Self::project_source_url] if it is the
    ///   same as the global [PackageMetadata::project_source_url] variable.
    /// - Remove the [package_source_url][Self::package_source_url] if it is the
    ///   same as the global [PackageMetadata::package_source_url] variable.
    /// - Remove the [license_url][Self::license_url] if it is the same as the
    ///   global [PackageMetadata::license] variable.
    /// - Remove the tag matching the automatically generated identifier.
    fn reset_same<R: AsRef<PackageMetadata>>(&mut self, from: R) {
        let from = from.as_ref();
        let id = self.id().to_lowercase();
        if self.id() == generate_identifier(from.id(), self.lowercase_id()) {
            self.id.clear();
        }

        if self.maintainers() == from.maintainers() {
            self.maintainers.clear();
        }

        if let Some(ref summary) = self.summary {
            if summary == &from.summary {
                self.summary = None;
            }
        }

        if let Some(ref url) = self.project_url {
            if url == &from.project_url {
                self.project_url = None;
            }
        }

        if let Some(ref url) = self.project_source_url {
            if url == from.project_source_url().as_ref() {
                self.project_source_url = None;
            }
        }

        if let Some(ref url) = self.package_source_url {
            if url == from.package_source_url().as_ref() {
                self.package_source_url = None;
            }
        }

        if let Some(ref url) = self.license_url {
            if let Some(license_url) = from.license().license_url() {
                if url.as_ref() == license_url {
                    self.license_url = None;
                }
            }
        }

        self.tags.retain(|t| t != &id);
    }
}

#[cfg(test)]
mod tests {

    use lazy_static::lazy_static;
    use rand::distributions::Alphanumeric;
    use rand::prelude::*;

    use super::*;

    const IDENTIFIER_NAMES: &[&str] = &[
        "7Zip",
        "CCleaner",
        "Google Chrome",
        "Firefox",
        "Chocolatey",
        "Pacman",
        "AER",
        "AU",
        "CCVARN",
        "GitReleaseManager",
    ];

    const MAINTAINERS: &[&str] = &[
        "AdmiringWorm",
        "gep13",
        "pauby",
        "chtof",
        "majkinetor",
        "mkevenaar",
        "ferventcoder",
        "TheCakeIsNaOH",
        "virtualex",
    ];

    const SPDIXES: &[&str] = &[
        "ADSL",
        "AGPL-1.0-only",
        "AGPL-3.0",
        "Apache-1.0",
        "Apache-2.0",
        "BSD-4-Clause",
        "CAL-1.0",
        "CC-BY-SA-4.0",
        "GPL-1.0+",
        "GPL-2.0+",
        "GPL-3.0",
        "GPL-3.0+",
        "GPL-3.0-only",
        "GPL-3.0-with-GCC-exception",
        "MIT-0",
        "SSH-OpenSSH",
    ];

    lazy_static! {
        static ref URLS: Vec<Url> = vec![
            Url::parse("https://chocolatey.org").unwrap(),
            Url::parse("https://github.com").unwrap(),
            Url::parse("https://github.com/WormieCorp/aer").unwrap(),
            Url::parse("https://bitbucket.org").unwrap()
        ];
    }

    fn get_maintainers(rng: &mut ThreadRng) -> Vec<&str> {
        let mnum = rng.gen_range(1..MAINTAINERS.len());
        get_maintainers_num(rng, mnum)
    }

    fn get_maintainers_num(rng: &mut ThreadRng, num: usize) -> Vec<&str> {
        MAINTAINERS.choose_multiple(rng, num).map(|m| *m).collect()
    }

    fn get_rand_alphanum(rng: &mut ThreadRng, len: usize) -> String {
        rng.sample_iter(Alphanumeric)
            .take(len)
            .map(char::from)
            .collect()
    }

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
            maintainers: Vec::new(),
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

    #[test]
    fn update_from_should_set_lowercase_identifier() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::with_id(&id, true);
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::new(id);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_lowercase_identifier() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let pkg = PackageMetadata::new(id);

        let mut actual = ChocolateyMetadata::with_id(id, true);
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_identifier_and_tag() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::with_id(&id, false);
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::new(id);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.lowercase_id = false;
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_identifier_and_tag() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.lowercase_id = false;
        let pkg = PackageMetadata::new(id);

        let mut actual = ChocolateyMetadata::with_id(id, false);
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_identifier() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::with_id(id, false);
        let pkg = PackageMetadata::new("Test Id");

        let mut actual = expected.clone();
        actual.reset_same(pkg);
        expected.tags.clear();

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_identifier() {
        let mut rng = thread_rng();
        let id = *IDENTIFIER_NAMES.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::with_id(&id, false);
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::new("Test value");
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::with_id(id, false);
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_maintainers() {
        let mut rng = thread_rng();
        let maintainers = get_maintainers(&mut rng);
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.maintainers = maintainers.iter().map(|m| m.to_string()).collect();
        let mut pkg = PackageMetadata::default();
        pkg.set_maintainers(&maintainers);

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_maintainers() {
        let mut rng = thread_rng();
        let maintainers = get_maintainers(&mut rng);
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.maintainers = maintainers.iter().map(|m| m.to_string()).collect();
        let mut pkg = PackageMetadata::default();
        pkg.set_maintainers(&["Test User 1", "Test User 2"]);

        let mut actual = expected.clone();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_maintainers() {
        let mut rng = thread_rng();
        let maintainers = get_maintainers(&mut rng);
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_maintainers(&maintainers);

        let mut actual = ChocolateyMetadata::default();
        actual.maintainers = pkg.maintainers.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_maintainers() {
        let mut rng = thread_rng();
        let maintainers = get_maintainers(&mut rng);
        let mut expected = ChocolateyMetadata::default();
        expected.maintainers = maintainers.iter().map(|m| m.to_string()).collect();
        let mut pkg = PackageMetadata::default();
        pkg.set_maintainers(&["Test Maint 1", "Test Maint 2"]);

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_summary() {
        let mut rng = thread_rng();
        let summary = get_rand_alphanum(&mut rng, 50);
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.summary = Some(summary.clone());
        let mut pkg = PackageMetadata::default();
        pkg.summary = summary;
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_set_empty_summary() {
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_summary() {
        let mut rng = thread_rng();
        let summary = get_rand_alphanum(&mut rng, 50);
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.summary = Some(summary);
        let mut pkg = PackageMetadata::default();
        pkg.summary = get_rand_alphanum(&mut rng, 40);
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_summary() {
        let mut rng = thread_rng();
        let summary = get_rand_alphanum(&mut rng, 50);
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.summary = summary.clone();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.summary = Some(summary);
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_summary() {
        let mut rng = thread_rng();
        let mut expected = ChocolateyMetadata::default();
        expected.summary = Some(get_rand_alphanum(&mut rng, 40));
        let mut pkg = PackageMetadata::default();
        pkg.summary = get_rand_alphanum(&mut rng, 20);
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_project_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_url(url);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_project_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_url("https://test-replace.not");
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.project_url = Some(url.clone());
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_project_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_project_url(url);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.project_url = Some(url.clone());
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_project_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_url("https://test-replace.not");
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_project_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.project_source_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_source_url(url).unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_project_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_source_url = Some(url.clone());
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_source_url("https://test-replace.not")
            .unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.project_source_url = Some(url.clone());
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_project_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_project_source_url(url).unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.project_source_url = Some(url.clone());
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_project_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_source_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_project_source_url("https://test-replace.not")
            .unwrap();
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_package_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.package_source_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_package_source_url(url).unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_package_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.package_source_url = Some(url.clone());
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_package_source_url("https://test-replace.not")
            .unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.package_source_url = Some(url.clone());
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_package_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_package_source_url(url).unwrap();
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.package_source_url = Some(url.clone());
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_package_source_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.package_source_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_package_source_url("https://test-replace.not")
            .unwrap();
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_license_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.license_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_license(url);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_license_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(url.clone());
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_license("https://test-replace.not");
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url.clone());
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_license_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_license(url);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url.clone());
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_license_url() {
        let mut rng = thread_rng();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_license("https://test-replace.not");
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_license_expression() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = Url::parse(
            LicenseType::Expression(spdix.to_string())
                .license_url()
                .unwrap(),
        )
        .unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.license_url = Some(url);
        let mut pkg = PackageMetadata::default();
        pkg.set_license(spdix);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_set_license_on_unsupported_expression() {
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_license("invalid-spdix");
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_license_expression() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = Url::parse(
            LicenseType::Expression(spdix.to_string())
                .license_url()
                .unwrap(),
        )
        .unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(url.clone());
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_license(spdix);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url);
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_license_expression() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = Url::parse(
            LicenseType::Expression(spdix.to_string())
                .license_url()
                .unwrap(),
        )
        .unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_license(spdix);
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url);
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_license_expression() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = crate::defaults::url();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(url);
        let mut pkg = PackageMetadata::default();
        pkg.set_license(spdix);
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_set_expected_license_expression_and_location() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.project_url = Some(crate::defaults::url());
        expected.license_url = Some(url.clone());
        let mut pkg = PackageMetadata::default();
        pkg.set_license((spdix, url));
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn update_from_should_not_replace_existing_license_expression_and_location() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(url.clone());
        expected.project_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_license((spdix, url));
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url.clone());
        actual.update_from(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_remove_same_license_expression_and_location() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = URLS.choose(&mut rng).unwrap();
        let expected = ChocolateyMetadata::default();
        let mut pkg = PackageMetadata::default();
        pkg.set_license((spdix, url));
        pkg.maintainers.clear();

        let mut actual = ChocolateyMetadata::default();
        actual.license_url = Some(url.clone());
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }

    #[test]
    fn reset_same_should_not_remove_different_license_expression_and_location() {
        let mut rng = thread_rng();
        let spdix = *SPDIXES.choose(&mut rng).unwrap();
        let url = URLS.choose(&mut rng).unwrap();
        let mut expected = ChocolateyMetadata::default();
        expected.license_url = Some(crate::defaults::url());
        let mut pkg = PackageMetadata::default();
        pkg.set_license((spdix, url));
        pkg.maintainers.clear();

        let mut actual = expected.clone();
        actual.reset_same(pkg);

        assert_eq!(actual, expected);
    }
}
