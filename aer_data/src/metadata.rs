// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#[cfg(feature = "chocolatey")]
pub mod chocolatey;

use std::borrow::Cow;
use std::fmt::Display;
use std::path::PathBuf;

use aer_license::LicenseType;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize), serde(untagged))]
pub enum Description {
    None,
    Location {
        from: PathBuf,
        skip_start: u16,
        skip_end: u16,
    },
    Text(String),
}

impl PartialEq<str> for Description {
    fn eq(&self, right: &str) -> bool {
        self == &Description::Text(right.into())
    }
}

/// Stores common values that are related to 1 or more package managers.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct PackageMetadata {
    /// The identifier of the package
    id: String,

    /// The list of maintainers that are responsible for the creating and
    /// maintaining of the package(s).
    #[cfg_attr(feature = "serialize", serde(default = "crate::defaults::maintainer"))]
    maintainers: Vec<String>,

    /// The main enpoints (homepage) of the software.
    pub summary: String,

    /// The main endpoint (homepage) of the software.
    project_url: Url,

    /// The type of the license, this can be either a supported expression (Like
    /// `MIT`, `GPL`, etc.) or an url the location of the license.
    ///
    /// ### Examples
    ///
    /// A `TOML` edition of only specifying a License Expression.
    /// ```toml
    /// [metadata]
    /// id = "test-package"
    /// project_url = "https://some-page.org"
    /// license = "MIT"
    /// ```
    ///
    /// A `TOML` edition of only specifying a License URL.
    /// ```toml
    /// [metadata]
    /// id = "test-package"
    /// project_url = "https://some-page.org"
    /// license = "https://some-page.org/license"
    /// ```
    ///
    /// A `TOML` edition of specifying both a License Expression and a License.
    /// This edition is recommended in most cases when creating packages for
    /// multiple package managers. URL.
    /// ```toml
    /// [metadata]
    /// id = "test-package"
    /// project_url = "https://some-page.org"
    /// license = { expression = "MIT", location = "https://some-page.org/license" }
    /// ```
    ///
    /// ### Notes
    ///
    /// If creating a chocolatey package, a license url is usually necessary
    /// when pushing to the chocolatey repository.
    #[cfg_attr(feature = "serialize", serde(default))]
    license: LicenseType,

    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    chocolatey: Option<chocolatey::ChocolateyMetadata>,
}

impl PackageMetadata {
    /// Creates a new instance of the package metadata with the specified
    /// identifier.
    pub fn new(id: &str) -> PackageMetadata {
        PackageMetadata {
            id: id.to_owned(),
            maintainers: crate::defaults::maintainer(),
            summary: String::new(),
            project_url: Url::parse("https://example-repo.org").unwrap(),
            license: LicenseType::None,
            #[cfg(feature = "chocolatey")]
            chocolatey: None,
        }
    }

    /// Returns the main identifier for the package.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns wether metadata regarding chocolatey is already set or not.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn has_chocolatey(&self) -> bool {
        self.chocolatey.is_some()
    }

    /// Returns the set chocolatey metadata, or a new instance if no data is
    /// set.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn chocolatey(&self) -> Cow<chocolatey::ChocolateyMetadata> {
        if let Some(ref choco) = self.chocolatey {
            Cow::Borrowed(choco)
        } else {
            Cow::Owned(chocolatey::ChocolateyMetadata::new())
        }
    }

    /// Returns the people responsible for creating and updating the package.
    pub fn maintainers(&self) -> &[String] {
        self.maintainers.as_slice()
    }

    /// Returns the url to the landing page of the software.
    pub fn project_url(&self) -> &Url {
        &self.project_url
    }

    /// Returns the license of the current software.
    pub fn license(&self) -> &LicenseType {
        &self.license
    }

    /// Allows setting a new instance of chocolatey metadata and associate it
    /// with the current metadata instance.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn set_chocolatey(&mut self, choco: chocolatey::ChocolateyMetadata) {
        self.chocolatey = Some(choco);
    }

    pub fn set_maintainers<T>(&mut self, vals: &[T])
    where
        T: Display,
    {
        let mut maintainers = Vec::<String>::with_capacity(vals.len());

        for val in vals.iter() {
            maintainers.push(val.to_string());
        }

        self.maintainers = maintainers;
    }

    pub fn set_project_url(&mut self, url: &str) {
        let url = Url::parse(url).unwrap(); // We want a failure here to abort the program
        self.project_url = url;
    }

    pub fn set_license(&mut self, license: LicenseType) {
        self.license = license;
    }
}

impl Default for PackageMetadata {
    fn default() -> PackageMetadata {
        PackageMetadata::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_create_default_metadata_with_expected_values() {
        let expected = PackageMetadata {
            id: "test-package".to_owned(),
            maintainers: crate::defaults::maintainer(),
            project_url: Url::parse("https://example-repo.org").unwrap(),
            license: LicenseType::None,
            summary: String::new(),
            #[cfg(feature = "chocolatey")]
            chocolatey: None,
        };

        let actual = PackageMetadata::new("test-package");

        assert_eq!(actual, expected);
    }

    #[test]
    fn default_should_create_default_metadata_with_expected_values() {
        let expected = PackageMetadata::new("");

        let actual = PackageMetadata::default();

        assert_eq!(actual, expected);
    }

    #[test]
    fn id_should_return_set_identifier() {
        const EXPECTED: &str = "my-awesome-test-package";

        let pkg = PackageMetadata::new(EXPECTED);

        assert_eq!(pkg.id(), EXPECTED);
    }

    #[test]
    fn maintainers_should_return_set_maintainers() {
        let expected = [
            "AdmiringWorm".to_owned(),
            "Some maintainer".to_owned(),
            "Some other".to_owned(),
        ];
        let mut pkg = PackageMetadata::new("test");
        pkg.maintainers = Vec::from(expected.clone());

        assert_eq!(pkg.maintainers(), expected);
    }

    #[test]
    fn project_url_should_return_set_project_url() {
        let expected = Url::parse("https://github.com/WormieCorp/aer").unwrap();
        let mut pkg = PackageMetadata::new("test");
        pkg.project_url = expected.clone();

        assert_eq!(pkg.project_url(), &expected);
    }

    #[cfg(feature = "chocolatey")]
    #[test]
    fn chocolatey_should_return_set_data() {
        let expected = chocolatey::ChocolateyMetadata::with_authors(&["AdmiringWorm", "kim"]);

        let mut data = PackageMetadata::new("some-id");
        data.set_chocolatey(expected.clone());

        assert!(data.has_chocolatey());
        assert_eq!(data.chocolatey(), Cow::Owned(expected));
    }

    #[cfg(feature = "chocolatey")]
    #[test]
    fn chocolatey_should_return_default_data() {
        let data = PackageMetadata::new("some-other-id");

        assert!(!data.has_chocolatey());
        assert_eq!(
            data.chocolatey(),
            Cow::Owned(chocolatey::ChocolateyMetadata::new())
        );
    }
}
