// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Collects different values that should be treated as the default,
//! in the case that [Default::default] is not the one we want.

#[cfg(feature = "chocolatey")]
use aer_version::{SemVersion, Versions};

/// Only used when serializing through serde and we want the defeult to be
/// [true].
#[cfg(feature = "serialize")]
pub fn boolean_true() -> bool {
    true
}

/// Since we do not allow unset versions, we need to create
/// a new default version.
/// This function creates a new [SemVersion] and sets the version to 0.0.0.
#[cfg(feature = "chocolatey")]
pub fn empty_version() -> Versions {
    Versions::SemVer(SemVersion::new(0, 0, 0))
}

/// The default value that should be set when a url is required.
/// This function will create a new url and set the value to `https://example.com/MUST_BE_CHANGED`.
pub fn url() -> url::Url {
    url::Url::parse("https://example.com/MUST_BE_CHANGED").unwrap()
}

/// The default maintainer of the package(s) if one is not already specified.
/// It will use the environment variable `AER_MAINTAINER` if it is already set,
/// or the current user in the operating system if the environment variable is
/// not available.
pub fn maintainer() -> Vec<String> {
    vec![match std::env::var("AER_MAINTAINER") {
        Ok(maintainer) => maintainer,
        Err(_) => whoami::username(),
    }]
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    /// This test is only added for coverage reasons.
    #[test]
    #[cfg(feature = "serialize")]
    fn boolean_true_should_be_true() {
        assert!(boolean_true());
    }

    #[test]
    #[cfg(feature = "chocolatey")]
    fn empty_version_should_set_version_to_zero() {
        assert_eq!(empty_version(), Versions::SemVer(SemVersion::new(0, 0, 0)))
    }

    #[test]
    fn url_should_set_default_url() {
        assert_eq!(
            url(),
            Url::parse("https://example.com/MUST_BE_CHANGED").unwrap()
        )
    }

    #[test]
    #[ignore = "Setting an environment variable is problematic when running tests in parallel!"]
    fn maintainer_should_set_maintainer_from_environment() {
        std::env::set_var("AER_MAINTAINER", "The Maintainer");

        assert_eq!(maintainer(), &["The Maintainer"]);

        std::env::remove_var("AER_MAINTAINER");
    }

    #[test]
    fn maintainer_should_be_set_from_operating_system_user() {
        let expected = whoami::username();

        assert_eq!(maintainer(), &[expected]);
    }
}
