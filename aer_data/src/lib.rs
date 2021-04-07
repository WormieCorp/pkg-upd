// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains data related to building packages.
//! This can be metadata for specific package managers, build scripts, urls and
//! version regexes and more.
//!
//! ### Examples
//!
//! The following is an example of a minimal configuration
//! file for a package in the `TOML` language.
//!
//! ```toml
//! [metadata]
//! id = "test-package"
//! version = "1.0.0"
//! project_url = "https://test.com/test-package"
//! summary = "Short summary of the software"
//!
//! [metadata.chocolatey]
//! lowercase_id = true
//! authors = ["AdmiringWorm"]
//! description = """\
//!     This is a multiline description \
//!     example of what will be included \
//!     in a Chocolatey package. \
//!     The ending \\ means that all whitespace wil be trimmed \
//! """
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

mod defaults;
pub mod metadata;
pub mod prelude;
pub mod updater;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Structure for holding all available data that a user can specify for a
/// package.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct PackageData {
    /// The metadata that will be part of any package that gets created.
    metadata: metadata::PackageMetadata,

    #[cfg_attr(feature = "serialize", serde(default))]
    updater: updater::PackageUpdateData,
}

impl PackageData {
    // Creates a new instance of a structure holding user data.
    pub fn new(id: &str) -> PackageData {
        PackageData {
            metadata: metadata::PackageMetadata::new(id),
            updater: updater::PackageUpdateData::new(),
        }
    }

    /// Returns the metadata available for this package.
    pub fn metadata(&self) -> &metadata::PackageMetadata {
        &self.metadata
    }

    /// Returns the metadata available for this package as a mutable reference.
    pub fn metadata_mut(&mut self) -> &mut metadata::PackageMetadata {
        &mut self.metadata
    }

    /// Returns the updater data available for this package.
    pub fn updater(&self) -> &updater::PackageUpdateData {
        &self.updater
    }

    /// Returns the updater data available for this package as a mutable
    /// reference.
    pub fn updater_mut(&mut self) -> &mut updater::PackageUpdateData {
        &mut self.updater
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_should_set_expected_values() {
        let expected = PackageData {
            metadata: metadata::PackageMetadata::new("test-id"),
            updater: updater::PackageUpdateData::new(),
        };

        let actual = PackageData::new("test-id");

        assert_eq!(actual, expected);
    }

    #[test]
    fn metadata_should_return_set_metadata() {
        let pkg_create = || {
            let mut pkg = metadata::PackageMetadata::new("test-id");
            pkg.set_license(aer_license::LicenseType::Expression("MIT".to_owned()));
            pkg
        };
        let pkg = PackageData {
            metadata: pkg_create(),
            updater: updater::PackageUpdateData::new(),
        };

        let actual = pkg.metadata();

        assert_eq!(actual, &pkg_create());
    }
}
