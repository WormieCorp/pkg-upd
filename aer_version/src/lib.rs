// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
#![cfg_attr(docsrs, feature(doc_cfg))]

mod versions;

use std::error::Error;
use std::fmt::Display;

pub use semver::Version as SemVersion;
#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "chocolatey")]
pub use versions::chocolatey;
pub use versions::FixVersion;

#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize), serde(untagged))]
#[derive(Debug, Clone, PartialEq)]
pub enum Versions {
    SemVer(SemVersion),
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    Choco(chocolatey::ChocoVersion),
}

/// An error type for this crate
///
/// Currently, just a generic error.
#[derive(Clone, PartialEq, Debug, PartialOrd)]
pub enum SemanticVersionError {
    /// An error occurred while parsing.
    ParseError(String),
}

impl Display for SemanticVersionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            SemanticVersionError::ParseError(ref m) => write!(f, "{}", m),
        }
    }
}

impl Error for SemanticVersionError {}

impl Versions {
    pub fn parse(val: &str) -> Result<Versions, Box<dyn std::error::Error>> {
        #[cfg(not(feature = "chocolatey"))]
        {
            Ok(Versions::SemVer(SemVersion::parse(val)?))
        }
        #[cfg(feature = "chocolatey")]
        {
            if let Ok(semver) = SemVersion::parse(val) {
                Ok(Versions::SemVer(semver))
            } else {
                let val = chocolatey::ChocoVersion::parse(val)?;
                Ok(Versions::Choco(val))
            }
        }
    }

    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn to_choco(&self) -> chocolatey::ChocoVersion {
        match self {
            Versions::SemVer(semver) => chocolatey::ChocoVersion::from(semver.clone()),
            Versions::Choco(ver) => ver.clone(),
        }
    }

    pub fn to_semver(&self) -> SemVersion {
        match self {
            Versions::SemVer(semver) => semver.clone(),
            #[cfg(feature = "chocolatey")]
            #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
            Versions::Choco(ver) => SemVersion::from(ver.clone()),
        }
    }
}

impl Display for Versions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Versions::SemVer(version) => version.fmt(f),
            #[cfg(feature = "chocolatey")]
            Versions::Choco(version) => version.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    #[cfg(feature = "chocolatey")]
    fn parse_should_use_chocolatey_version_on_4_part_versions() {
        let expected = Versions::Choco(chocolatey::ChocoVersion::with_build(5, 1, 6, 4));
        let version = Versions::parse("5.1.6.4").unwrap();

        assert_eq!(version, expected);
    }

    #[test]
    fn parse_should_use_semver_version_on_3_part_versions() {
        let expected = Versions::SemVer(SemVersion::new(5, 1, 0));
        let version = Versions::parse("5.1.0").unwrap();

        assert_eq!(version, expected);
    }

    #[test]
    #[cfg_attr(
        feature = "chocolatey",
        should_panic(expected = "The version string do not start with a number")
    )]
    #[cfg_attr(
        not(feature = "chocolatey"),
        should_panic(expected = "encountered unexpected token: AlphaNumeric")
    )]
    fn parse_should_return_error_on_invalid_version() {
        Versions::parse("invalid").unwrap();
    }

    #[test]
    #[cfg_attr(
        feature = "chocolatey",
        should_panic(
            expected = "There were additional numeric characters after the first 4 parts of the \
                        version"
        )
    )]
    #[cfg_attr(
        not(feature = "chocolatey"),
        should_panic(expected = "expected end of input, but got:")
    )]
    fn parse_should_return_error_on_5_part_version() {
        // This may be valid at a later date, if/when python support is added
        Versions::parse("2.0.2.5.1").unwrap();
    }

    #[test]
    #[cfg(feature = "chocolatey")]
    fn to_semver_should_create_semversion_from_choco_version() {
        let version =
            Versions::Choco(chocolatey::ChocoVersion::parse("2.1.0.5-alpha0055").unwrap());
        let expected = SemVersion::parse("2.1.0-alpha.55+5").unwrap();

        let actual = version.to_semver();

        assert_eq!(actual, expected);
    }

    #[test]
    fn to_semver_should_return_cloned_version_of_semver() {
        let version = Versions::SemVer(SemVersion::parse("5.2.2-alpha.5+55").unwrap());
        let expected = SemVersion::parse("5.2.2-alpha.5+55").unwrap();

        let actual = version.to_semver();

        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg(feature = "chocolatey")]
    fn to_choco_should_create_chocolatey_version_from_semver() {
        let version = Versions::SemVer(SemVersion::parse("1.0.5-beta.55+99").unwrap());
        let expected = chocolatey::ChocoVersion::parse("1.0.5-beta0055").unwrap();

        let actual = version.to_choco();

        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg(feature = "chocolatey")]
    fn to_choco_should_returned_cloned_version_of_choco() {
        let version =
            Versions::Choco(chocolatey::ChocoVersion::parse("5.2.1.56-unstable-0050").unwrap());
        let expected = chocolatey::ChocoVersion::parse("5.2.1.56-unstable0050").unwrap();

        let actual = version.to_choco();

        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg(feature = "chocolatey")]
    fn display_choco_version() {
        let version =
            Versions::Choco(chocolatey::ChocoVersion::parse("2.1.0-unstable-0050").unwrap());
        let expected = "2.1.0-unstable0050";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[rstest]
    #[case("4.2.1-alpha.5+6", "4.2.1-alpha.5+6")]
    #[cfg_attr(feature = "chocolatey", case("3.2", "3.2"))]
    #[cfg_attr(feature = "chocolatey", case("5.2.1.6-beta-0005", "5.2.1.6-beta0005"))]
    fn display_version(#[case] test: &str, #[case] expected: &str) {
        let version = Versions::parse(test).unwrap();

        assert_eq!(version.to_string(), expected);
    }
}
