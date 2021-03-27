// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use pkg_version::{SemVersion, Versions};

pub fn boolean_true() -> bool {
    true
}

pub fn empty_version() -> Versions {
    Versions::SemVer(SemVersion::new(0, 0, 0))
}

pub fn maintainer() -> Vec<String> {
    vec![match std::env::var("PKG_MAINTAINER") {
        Ok(maintainer) => maintainer,
        Err(_) => whoami::username(),
    }]
}
