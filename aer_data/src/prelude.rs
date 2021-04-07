// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

pub use aer_license::LicenseType;
pub use aer_version::{FixVersion, SemVersion, Versions};
pub use url::Url;

pub use crate::metadata::{Description, PackageMetadata};
pub use crate::updater::PackageUpdateData;
pub use crate::PackageData;

/// Re-Exports of usable chocolatey types.
#[cfg(feature = "chocolatey")]
#[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
pub mod chocolatey {
    pub use aer_version::chocolatey::ChocoVersion;

    pub use crate::metadata::chocolatey::ChocolateyMetadata;
    pub use crate::updater::chocolatey::{
        ChocolateyParseUrl, ChocolateyUpdaterData, ChocolateyUpdaterType,
    };
}
