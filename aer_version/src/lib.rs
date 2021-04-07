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
    use super::*;

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

        let actual = format!("{}", version);

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_semver() {
        let version = Versions::SemVer(SemVersion::parse("2.5.2+build.50").unwrap());
        let expected = "2.5.2+build.50";

        let actual = format!("{}", version);

        assert_eq!(actual, expected);
    }
}
