// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! The data stored that can be used by different package manager updaters.

pub mod chocolatey;

#[cfg(feature = "chocolatey")]
use std::borrow::Cow;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

/// Collection of different package managers that will be used when updating
/// packages (when possible). Additionally, plans for adding hooks/scripts are
/// in the works.
#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct PackageUpdateData {
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    chocolatey: Option<chocolatey::ChocolateyUpdaterData>,
}

impl PackageUpdateData {
    /// Creates a new instance of the [PackageUpdateData] struct with the values
    /// set to default.
    pub fn new() -> PackageUpdateData {
        PackageUpdateData::default()
    }

    /// Returns wether data regarding chocolatey is already set for the updater.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn has_chocolatey(&self) -> bool {
        self.chocolatey.is_some()
    }

    /// Returns the current set updater data, or a new instance if no data is
    /// already set.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn chocolatey(&self) -> Cow<chocolatey::ChocolateyUpdaterData> {
        if let Some(ref chocolatey) = self.chocolatey {
            Cow::Borrowed(chocolatey)
        } else {
            Cow::Owned(chocolatey::ChocolateyUpdaterData::new())
        }
    }

    /// Allows associating new updater data with the current instance.
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    pub fn set_chocolatey(&mut self, choco: chocolatey::ChocolateyUpdaterData) {
        self.chocolatey = Some(choco);
    }
}

#[cfg(all(test, feature = "chocolatey"))]
mod tests {
    use super::*;

    #[test]
    fn should_get_set_chocolatey_data() {
        let mut expected = chocolatey::ChocolateyUpdaterData::new();
        expected.add_regex("arch32", "MY REGEX");

        let mut data = PackageUpdateData::new();
        data.set_chocolatey(expected.clone());

        assert!(data.has_chocolatey());
        assert_eq!(data.chocolatey(), Cow::Owned(expected));
    }

    #[test]
    fn should_return_default_chocolatey() {
        let expected = chocolatey::ChocolateyUpdaterData::new();

        let data = PackageUpdateData::new();
        assert!(!data.has_chocolatey());
        assert_eq!(data.chocolatey(), Cow::Owned(expected));
    }
}
