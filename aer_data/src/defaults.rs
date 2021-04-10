// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#[cfg(feature = "chocolatey")]
use aer_version::{SemVersion, Versions};

#[cfg(feature = "chocolatey")]
pub fn boolean_true() -> bool {
    true
}

#[cfg(feature = "chocolatey")]
pub fn empty_version() -> Versions {
    Versions::SemVer(SemVersion::new(0, 0, 0))
}

pub fn url() -> url::Url {
    url::Url::parse("https://example.com/MUST_BE_CHANGED").unwrap()
}

pub fn maintainer() -> Vec<String> {
    vec![match std::env::var("AER_MAINTAINER") {
        Ok(maintainer) => maintainer,
        Err(_) => whoami::username(),
    }]
}
