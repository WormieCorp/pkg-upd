// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

pub mod chocolatey;

use std::borrow::Cow;

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize))]
#[non_exhaustive]
pub struct PackageUpdateData {
    #[cfg(feature = "chocolatey")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]
    chocolatey: Option<chocolatey::ChocolateyUpdaterData>,
}

impl PackageUpdateData {
    pub fn new() -> PackageUpdateData {
        PackageUpdateData {
            #[cfg(feature = "chocolatey")]
            chocolatey: None,
        }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "chocolatey")]
    #[test]
    fn should_get_set_chocolatey_data() {
        let mut expected = chocolatey::ChocolateyUpdaterData::new();
        expected.add_regex("arch32", "MY REGEX");

        let mut data = PackageUpdateData::new();
        data.set_chocolatey(expected.clone());

        assert!(data.has_chocolatey());
        assert_eq!(data.chocolatey(), Cow::Owned(expected));
    }

    #[cfg(feature = "chocolatey")]
    #[test]
    fn should_return_default_chocolatey() {
        let expected = chocolatey::ChocolateyUpdaterData::new();

        let data = PackageUpdateData::new();
        assert!(!data.has_chocolatey());
        assert_eq!(data.chocolatey(), Cow::Owned(expected));
    }
}
