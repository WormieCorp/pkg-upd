// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#[cfg(feature = "serialize")]
use serde::{Deserialize, Serialize};
use url::Url;

/// The type or location of the license for the packaged software.
#[derive(Debug, PartialEq)]
#[cfg_attr(feature = "serialize", derive(Deserialize, Serialize), serde(untagged))]
pub enum LicenseType {
    /// When there are no License available at all.
    /// The item should in general never be used, but is provided for
    /// convenience.
    None,
    /// The remote location of an url, this can be used when there is no
    /// expression available for the package you want to create.
    /// Depending on the package created the license may get downloaded during
    /// updated and embedded in the package.
    Location(Url),
    /// Allows specifying an expression of the License Type to use for the
    /// package.
    ///
    /// ### Notes
    ///
    /// No validation is done on this expression, and it is your responsibility
    /// to ensure the expression is valid for the packages that you are
    /// creating.
    Expression(String),
    /// Allows specifying both the expression and the remote location of a
    /// license. The item is preferred to be used when targeting multiple
    /// package managers.
    ExpressionAndLocation {
        /// Allows specifying an expression of the License Type to use for the
        /// package.
        expression: String,
        /// The remote location of an url
        url: Url,
    },
}

impl Default for LicenseType {
    fn default() -> Self {
        Self::None
    }
}

impl LicenseType {
    pub fn license_url(&self) -> Option<&str> {
        match self {
            LicenseType::Location(url) | LicenseType::ExpressionAndLocation { url, .. } => {
                Some(url.as_str())
            }
            LicenseType::Expression(expression) => {
                let resolved = license::from_id_exception(&expression);
                if let Some(license) = resolved {
                    if !license.see_also().is_empty() {
                        return Some(license.see_also()[0]);
                    }
                }
                let resolved = license::from_id_ext(&expression);
                if let Some(license) = resolved {
                    if !license.see_also().is_empty() {
                        return Some(license.see_also()[0]);
                    }
                }
                let resolved = license::from_id(&expression);
                if let Some(license) = resolved {
                    if !license.see_also().is_empty() {
                        return Some(license.see_also()[0]);
                    }
                }

                None
            }

            _ => None,
        }
    }
}

impl From<Url> for LicenseType {
    fn from(url: Url) -> Self {
        LicenseType::Location(url)
    }
}

impl From<&Url> for LicenseType {
    fn from(url: &Url) -> Self {
        LicenseType::Location(url.clone())
    }
}

impl From<String> for LicenseType {
    fn from(val: String) -> Self {
        if let Ok(url) = Url::parse(&val) {
            LicenseType::Location(url)
        } else {
            LicenseType::Expression(val)
        }
    }
}

impl From<&str> for LicenseType {
    fn from(val: &str) -> Self {
        val.to_owned().into()
    }
}

impl From<(&str, &Url)> for LicenseType {
    fn from(val: (&str, &Url)) -> Self {
        LicenseType::ExpressionAndLocation {
            expression: val.0.to_owned(),
            url: val.1.to_owned(),
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn license_url_should_return_url_set_with_location() {
        let license = LicenseType::Location(
            Url::parse("https://github.com/cake-contrib/Cake.Warp/blob/develop/LICENSE").unwrap(),
        );

        assert_eq!(
            license.license_url(),
            Some("https://github.com/cake-contrib/Cake.Warp/blob/develop/LICENSE")
        );
    }

    #[test]
    fn license_url_should_return_url_set_with_expression_and_url() {
        let license = LicenseType::ExpressionAndLocation {
            url: Url::parse("https://github.com/cake-contrib/Cake.Warp/blob/develop/LICENSE")
                .unwrap(),
            expression: "MIT".into(),
        };

        assert_eq!(
            license.license_url(),
            Some("https://github.com/cake-contrib/Cake.Warp/blob/develop/LICENSE")
        )
    }

    #[test]
    #[allow(non_snake_case)]
    fn license_url_should_None_when_license_is_not_set() {
        let license = LicenseType::None;

        assert_eq!(license.license_url(), None);
    }

    #[test]
    #[allow(non_snake_case)]
    fn license_url_should_return_None_on_unknown_license_type() {
        let license = LicenseType::Expression("Custom".into());

        assert_eq!(license.license_url(), None);
    }

    #[rstest(
        expression,
        url,
        case("Apache-2.0", "http://www.apache.org/licenses/LICENSE-2.0"),
        case("BSD-3-Clause", "https://opensource.org/licenses/BSD-3-Clause"),
        case("BSD-2-Clause", "https://opensource.org/licenses/BSD-2-Clause"),
        case(
            "GPL-2.0",
            "https://www.gnu.org/licenses/old-licenses/gpl-2.0-standalone.html"
        ),
        case(
            "GPL-2.0-only",
            "https://www.gnu.org/licenses/old-licenses/gpl-2.0-standalone.html"
        ),
        case("GPL-3.0", "https://www.gnu.org/licenses/gpl-3.0-standalone.html"),
        case(
            "LGPL-2.0",
            "https://www.gnu.org/licenses/old-licenses/lgpl-2.0-standalone.html"
        ),
        case(
            "LGPL-2.1",
            "https://www.gnu.org/licenses/old-licenses/lgpl-2.1-standalone.html"
        ),
        case("LGPL-3.0", "https://www.gnu.org/licenses/lgpl-3.0-standalone.html"),
        case("MIT", "https://opensource.org/licenses/MIT"),
        case("MPL-2.0", "http://www.mozilla.org/MPL/2.0/"),
        case("CDDL-1.0", "https://opensource.org/licenses/cddl1"),
        case("EPL-2.0", "https://www.eclipse.org/legal/epl-2.0"),
        case("GPL-3.0+", "https://www.gnu.org/licenses/gpl-3.0-standalone.html"),
        case(
            "BSD-3-Clause-No-Nuclear-License-2014",
            "https://java.net/projects/javaeetutorial/pages/BerkeleyLicense"
        ),
        case(
            "GCC-exception-3.1",
            "http://www.gnu.org/licenses/gcc-exception-3.1.html"
        )
    )]
    fn license_url_should_return_correct_license_url_for_expression(expression: &str, url: &str) {
        let license = LicenseType::Expression(expression.into());

        assert_eq!(license.license_url(), Some(url));
    }
}
