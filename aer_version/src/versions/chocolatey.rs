// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#![cfg(feature = "chocolatey")]
#![cfg_attr(docsrs, doc(cfg(feature = "chocolatey")))]

use std::cmp::Ordering;
use std::fmt::Display;

use semver::Identifier;
#[cfg(feature = "serialize")]
use serde::de::{self, Deserialize, Deserializer, Visitor};
#[cfg(feature = "serialize")]
use serde::ser::{Serialize, Serializer};

use crate::{FixVersion, SemVersion, SemanticVersionError};

#[derive(Default, Debug, Clone, Eq)]
pub struct ChocoVersion {
    major: u8,
    minor: u8,
    patch: Option<u8>,
    /// The build part of the version, this is specified as an unsigned 32bit
    /// integer to allow fix versions.
    build: Option<u32>,
    pre_release: Vec<Identifier>,
}

impl ChocoVersion {
    pub fn new(major: u8, minor: u8) -> ChocoVersion {
        ChocoVersion {
            major,
            minor,
            ..Default::default()
        }
    }

    pub fn with_patch(major: u8, minor: u8, patch: u8) -> ChocoVersion {
        let mut choco = ChocoVersion::new(major, minor);
        choco.set_patch(patch);
        choco
    }

    pub fn with_build(major: u8, minor: u8, patch: u8, build: u32) -> ChocoVersion {
        let mut choco = ChocoVersion::with_patch(major, minor, patch);
        choco.set_build(build);
        choco
    }

    pub fn parse(val: &str) -> Result<ChocoVersion, Box<dyn std::error::Error>> {
        if val.is_empty() {
            return Err(Box::new(SemanticVersionError::ParseError(
                "There is no version string to parse".into(),
            )));
        } else if !val.chars().next().unwrap_or('.').is_digit(10) {
            return Err(Box::new(SemanticVersionError::ParseError(
                "The version string do not start with a number".into(),
            )));
        }

        let mut major = 0;
        let mut minor = 0;
        let mut patch = None;
        let mut build = None;
        let mut i = 0;
        let mut ver_str = String::new();

        for ch in val.chars() {
            if ch.is_digit(10) {
                ver_str.push(ch);
            } else if ch == '.' {
                match i {
                    0 => major = ver_str.parse()?,
                    1 => minor = ver_str.parse()?,
                    2 => patch = Some(ver_str.parse()?),
                    3 => build = Some(ver_str.parse()?),
                    _ => {
                        return Err(Box::new(SemanticVersionError::ParseError(
                            "There were additional numeric characters after the first 4 parts of \
                             the version"
                                .into(),
                        )));
                    }
                };

                i += 1;

                ver_str.clear();
            } else {
                break;
            }
        }

        if !ver_str.is_empty() {
            match i {
                0 => major = ver_str.parse()?,
                1 => minor = ver_str.parse()?,
                2 => patch = Some(ver_str.parse()?),
                3 => build = Some(ver_str.parse()?),
                _ => {
                    return Err(Box::new(SemanticVersionError::ParseError(
                        "There were additional numeric characters after the first 4 parts of the \
                         version"
                            .into(),
                    )));
                }
            };
            ver_str.clear();
        }

        let subst: String = val
            .chars()
            .skip_while(|ch| ch.is_digit(10) || *ch == '.')
            .collect();

        let pre = { extract_prerelease(&subst) };

        let result = ChocoVersion {
            major,
            minor,
            patch,
            build,
            pre_release: pre,
        };

        Ok(result)
    }

    pub fn set_patch(&mut self, patch: u8) {
        self.patch = Some(patch);
    }

    pub fn set_build(&mut self, build: u32) {
        if self.patch.is_none() {
            self.patch = Some(0);
        }

        self.build = Some(build);
    }

    pub fn set_prerelease(&mut self, pre: Vec<Identifier>) {
        self.pre_release = pre;
    }

    pub fn with_prerelease(mut self, pre: Vec<Identifier>) -> Self {
        self.set_prerelease(pre);
        self
    }
}

impl Ord for ChocoVersion {
    fn cmp(&self, other: &Self) -> Ordering {
        let major_cmp = self.major.cmp(&other.major);
        if major_cmp != Ordering::Equal {
            return major_cmp;
        }
        let minor_cmp = self.minor.cmp(&other.minor);
        if minor_cmp != Ordering::Equal {
            return minor_cmp;
        }
        let patch_cmp = self.patch.unwrap_or(0).cmp(&other.patch.unwrap_or(0));
        if patch_cmp != Ordering::Equal {
            return patch_cmp;
        }
        let build_cmp = self.build.unwrap_or(0).cmp(&other.build.unwrap_or(0));
        if build_cmp != Ordering::Equal {
            return build_cmp;
        }

        self.pre_release.cmp(&other.pre_release)
    }
}

impl PartialOrd for ChocoVersion {
    fn partial_cmp(&self, other: &Self) -> std::option::Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl PartialEq for ChocoVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major
            && self.minor == other.minor
            && self.patch.unwrap_or(0) == other.patch.unwrap_or(0)
            && self.build.unwrap_or(0) == other.build.unwrap_or(0)
    }
}

impl FixVersion for ChocoVersion {
    fn add_fix(&mut self) -> Result<(), std::num::ParseIntError> {
        const THRESHOLD: u32 = 20070101;
        if self.build.unwrap_or(THRESHOLD) >= THRESHOLD {
            let fix = format!("{}", chrono::Local::today().format("%Y%m%d"));
            let num_fix = fix.parse()?;

            self.set_build(num_fix);
        }

        Ok(())
    }
}

impl From<SemVersion> for ChocoVersion {
    fn from(semver: SemVersion) -> Self {
        let mut choco = ChocoVersion::new(
            get_val(semver.major, u8::MAX as u64) as u8,
            get_val(semver.minor, u8::MAX as u64) as u8,
        );
        choco.set_patch(get_val(semver.patch, u8::MAX as u64) as u8);
        let mut pre_releases = vec![];
        for identifier in semver.pre {
            match identifier {
                Identifier::AlphaNumeric(val) => {
                    pre_releases.extend(extract_prerelease(&val));
                }
                Identifier::Numeric(val) => {
                    if pre_releases.is_empty() {
                        pre_releases.push(Identifier::AlphaNumeric("unstable".into()));
                    }
                    pre_releases.push(Identifier::Numeric(val));
                }
            }
        }

        choco.set_prerelease(pre_releases);

        choco
    }
}

impl From<ChocoVersion> for SemVersion {
    fn from(choco: ChocoVersion) -> Self {
        let mut ver_str = format!(
            "{}.{}.{}",
            choco.major,
            choco.minor,
            choco.patch.unwrap_or(0)
        );

        let mut build = 0;
        for pre in choco.pre_release {
            match pre {
                Identifier::AlphaNumeric(s) => {
                    let prefix: String = s.chars().take_while(|ch| !ch.is_digit(10)).collect();
                    let suffix: String = s.chars().skip_while(|ch| !ch.is_digit(10)).collect();
                    if !prefix.is_empty() {
                        ver_str.push_str(&format!("-{}", prefix));
                    }
                    if let Ok(num) = suffix.parse() {
                        if build > 0 {
                            ver_str.push_str(&format!("-{}", build));
                        }
                        build = num;
                    } else if !suffix.is_empty() {
                        if prefix.is_empty() {
                            ver_str.push_str(&format!("-{}", suffix));
                        } else {
                            ver_str.push_str(&suffix);
                        }
                    }
                }
                Identifier::Numeric(num) => {
                    if build > 0 {
                        ver_str.push_str(&format!("-{}", build));
                    }
                    build = num
                }
            }
        }

        let (delim, alt_delim) = if ver_str.contains('-') {
            ('.', '+')
        } else {
            ('+', '-')
        };

        if let Some(b) = choco.build {
            if build > 0 {
                ver_str.push_str(&format!("{}{}", delim, build));
                ver_str.push_str(&format!("{}{}", alt_delim, b));
            } else {
                ver_str.push_str(&format!("{}{}", delim, b));
            }
        } else if build > 0 {
            ver_str.push_str(&format!("{}{}", delim, build));
        }

        SemVersion::parse(&ver_str).unwrap()
    }
}

#[cfg(feature = "serialize")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialize")))]
impl Serialize for ChocoVersion {
    fn serialize<S>(&self, serialize: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Serialize ChocoVersion as a string
        serialize.collect_str(self)
    }
}

#[cfg(feature = "serialize")]
#[cfg_attr(docsrs, doc(cfg(feature = "serialize")))]
impl<'de> Deserialize<'de> for ChocoVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ChocoVersionVisitor;

        // Deserialize ChocoVersion from a string.
        impl<'de> Visitor<'de> for ChocoVersionVisitor {
            type Value = ChocoVersion;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a Chocolatey version as a string")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                ChocoVersion::parse(v).map_err(de::Error::custom)
            }
        }

        deserializer.deserialize_str(ChocoVersionVisitor)
    }
}

fn get_val<T: num::PrimInt>(value: T, max_value: T) -> T {
    if value > max_value { max_value } else { value }
}

fn extract_prerelease(val: &str) -> Vec<Identifier> {
    const NORMAL_PRE: &str = "unstable";
    let mut result = vec![];
    let mut current = String::new();
    let mut next = String::new();

    for ch in val.chars().take_while(|ch| *ch != '+') {
        if ch == '-' || ch == '.' {
            if let Some(res) = get_identifier(&current) {
                current.clear();
                result.push(res);
            }
            if let Some(res) = get_identifier(&next) {
                result.push(res);
                next.clear();
            }

            continue;
        } else if ch.is_digit(10) {
            if result.is_empty() && current.is_empty() {
                result.push(Identifier::AlphaNumeric(NORMAL_PRE.into()));
            } else if current.chars().any(|ch| !ch.is_digit(10)) {
                if let Some(res) = get_identifier(&current) {
                    current.clear();
                    result.push(res);
                }
            }
        } else if ch.is_digit(10) && result.is_empty() && current.is_empty() {
            result.push(Identifier::AlphaNumeric(NORMAL_PRE.into()));
        } else if !ch.is_digit(10)
            && current.is_empty()
            && result.len() > 1
            && result.first() == Some(&Identifier::AlphaNumeric(NORMAL_PRE.into()))
        {
            result.remove(0);
            next = result.pop().unwrap().to_string();
        } else if !ch.is_digit(10)
            && !current.is_empty()
            && result.first() == Some(&Identifier::AlphaNumeric(NORMAL_PRE.into()))
        {
            result.remove(0);
            next = current.clone();
            current.clear();
        }
        current.push(ch);
    }

    if let Some(res) = get_identifier(&current) {
        result.push(res);
    }

    if let Some(res) = get_identifier(&next) {
        result.push(res);
    }

    result
}

fn get_identifier(value: &str) -> Option<Identifier> {
    if value.is_empty() {
        return None;
    }

    if value.chars().all(|ch| ch.is_digit(10)) {
        if let Ok(num) = value.parse() {
            return Some(Identifier::Numeric(num));
        }
    }
    let (mut vals, mut nums) = (String::new(), String::new());

    for ch in value.chars() {
        if ch.is_digit(10) {
            nums.push(ch);
        } else {
            vals.push(ch);
        }
    }

    if nums.is_empty() {
        Some(Identifier::AlphaNumeric(vals))
    } else if let Ok(nums) = nums.parse::<u32>() {
        Some(Identifier::AlphaNumeric(format!("{}{:04}", vals, nums)))
    } else {
        Some(Identifier::AlphaNumeric(format!("{}{}", vals, nums)))
    }
}

impl Display for ChocoVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        write!(f, "{}.{}", self.major, self.minor)?;
        if let Some(patch) = self.patch {
            write!(f, ".{}", patch)?;
        }

        if let Some(build) = self.build {
            write!(f, ".{}", build)?;
        }

        let mut prev_alpha = false;

        for pre in &self.pre_release {
            match pre {
                Identifier::Numeric(num) => {
                    if prev_alpha {
                        write!(f, "{:04}", num)?;
                        prev_alpha = false;
                    } else {
                        write!(f, "-{:04}", num)?;
                    }
                }
                num => {
                    write!(f, "-{}", num)?;
                    prev_alpha = true;
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[test]
    fn display_should_output_major_and_minor_version() {
        let version = ChocoVersion::new(1, 2);
        let expected = "1.2";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_should_output_major_minor_and_patch_version() {
        let mut version = ChocoVersion::new(1, 6);
        version.set_patch(10);
        let expected = "1.6.10";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn display_should_output_major_minor_patch_and_build_version() {
        let mut version = ChocoVersion::new(0, 8);
        version.set_patch(3);
        version.set_build(99);
        let expected = "0.8.3.99";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn set_build_should_also_set_default_patch_number() {
        let mut version = ChocoVersion::new(1, 1);
        version.set_build(5);
        let expected = "1.1.0.5";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn with_build_should_set_full_build_version() {
        let version = ChocoVersion::with_build(5, 1, 1, 3);
        let expected = "5.1.1.3";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn add_fix_should_create_correct_fix_version() {
        let mut version = ChocoVersion::new(2, 1);
        version.add_fix().unwrap();
        let expected = format!("2.1.0.{}", chrono::Local::now().format("%Y%m%d"));

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn add_fix_should_not_create_fix_version_when_build_is_in_use() {
        let mut version = ChocoVersion::new(0, 2);
        version.set_build(5);
        version.add_fix().unwrap();
        let expected = "0.2.0.5";

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[test]
    fn add_fix_should_replace_old_date_fix_with_new_date() {
        let mut version = ChocoVersion::new(3, 3);
        version.set_build(20200826);
        version.add_fix().unwrap();
        let expected = format!("3.3.0.{}", chrono::Local::now().format("%Y%m%d"));

        let actual = version.to_string();

        assert_eq!(actual, expected);
    }

    #[rstest(
        v,
        expected,
        case("3", "3.0"),
        case("1.0", "1.0"),
        case("0.2.65", "0.2.65"),
        case("3.5.0.2342", "3.5.0.2342"),
        case("3.3-alpha001", "3.3-alpha0001"),
        case("3.2-alpha.10", "3.2-alpha0010"),
        case("3.3.5-beta-11", "3.3.5-beta0011"),
        case("3.1.1+55", "3.1.1"),
        case("4.0.0.2-beta.5", "4.0.0.2-beta0005"),
        case("0.1.0-55", "0.1.0-unstable0055"),
        case("4.2.1-alpha54.2", "4.2.1-alpha0054-0002"),
        case("6.1.0-55-alpha", "6.1.0-alpha0055")
    )]
    fn parse_should_create_correct_versions(v: &str, expected: &str) {
        let version = ChocoVersion::parse(v).unwrap();
        let version = version.to_string();

        assert_eq!(version, expected);
    }

    #[rstest(
        val,
        case(""),
        case("6.2.2.2.1"),
        case("no-version"),
        case("6.2.1.1.3.4")
    )]
    #[should_panic]
    fn parse_should_return_none(val: &str) {
        let _ = ChocoVersion::parse(val).unwrap();
    }

    #[test]
    fn from_should_create_choco_version() {
        let expected = ChocoVersion {
            major: 5,
            minor: 3,
            patch: Some(1),
            ..Default::default()
        };

        let actual = ChocoVersion::from(SemVersion::parse("5.3.1").unwrap());

        assert_eq!(actual, expected);
    }

    #[rstest(test, expected,
        case(
            "1.255.3-alpha+446",
            ChocoVersion::with_patch(1, 255, 3).with_prerelease(vec![Identifier::AlphaNumeric("alpha".into())])
        ),
        case(
            "0.3.0-unstable-001",
            ChocoVersion::with_patch(0, 3, 0).with_prerelease(vec![Identifier::AlphaNumeric("unstable".into()), Identifier::Numeric(1)])
        ),
        case(
            "5.1.1-alpha.5",
            ChocoVersion::with_patch(5, 1, 1).with_prerelease(vec![Identifier::AlphaNumeric("alpha".into()), Identifier::Numeric(5)])
        ),
        case(
            "1.2.3-beta50",
            ChocoVersion::with_patch(1, 2, 3).with_prerelease(vec![Identifier::AlphaNumeric("beta".into()), Identifier::Numeric(50)])
        ),
        case(
            "3.0.0-666",
            ChocoVersion::with_patch(3, 0, 0).with_prerelease(vec![Identifier::AlphaNumeric("unstable".into()), Identifier::Numeric(666)])
        ),
        case("2.0.0-55beta", ChocoVersion::with_patch(2, 0, 0).with_prerelease(vec![Identifier::AlphaNumeric("beta".into()), Identifier::Numeric(55)])),
        case("4.2.1-alpha54.2", ChocoVersion::with_patch(4, 2, 1).with_prerelease(vec![Identifier::AlphaNumeric("alpha".into()), Identifier::Numeric(54), Identifier::Numeric(2)])),
        case("6.1.0-55-alpha", ChocoVersion::with_patch(6, 1, 0).with_prerelease(vec![Identifier::AlphaNumeric("alpha".into()).into(), Identifier::Numeric(55)]))
    )]
    fn from_should_create_choco_version_with_prerelease(test: &str, expected: ChocoVersion) {
        let actual = ChocoVersion::from(SemVersion::parse(test).unwrap());

        assert_eq!(actual, expected);
    }

    #[rstest(
        test, expected,
        case("3.0.0-beta-0050", SemVersion::parse("3.0.0-beta.50").unwrap()),
        case("1.2.2.5-unstable-0050", SemVersion::parse("1.2.2-unstable.50+5").unwrap()),
        case("5.1-beta0995", SemVersion::parse("5.1.0-beta.995").unwrap()),
        case("1.0-alpha-0002-rc0005", SemVersion::parse("1.0.0-alpha-rc-2.5").unwrap()), // This ending version is due to chocolatey parsing
        case("5.0-beta-ceta", SemVersion::parse("5.0.0-beta-ceta").unwrap())
    )]
    fn from_should_create_sematic_version(test: &str, expected: SemVersion) {
        let actual = SemVersion::from(ChocoVersion::parse(test).unwrap());

        assert_eq!(actual, expected);
    }

    #[test]
    fn should_sort_versions() {
        let mut versions = vec![
            ChocoVersion::parse("1.2.0-55").unwrap(),
            ChocoVersion::parse("1.2").unwrap(),
            ChocoVersion::parse("0.4.2.1").unwrap(),
            ChocoVersion::parse("6.2.0").unwrap(),
            ChocoVersion::parse("1.0.0-rc").unwrap(),
            ChocoVersion::parse("1.0.0-alpha").unwrap(),
            ChocoVersion::parse("5.0-beta.56").unwrap(),
            ChocoVersion::parse("5.0-beta.55").unwrap(),
        ];
        let expected = vec![
            ChocoVersion::parse("0.4.2.1").unwrap(),
            ChocoVersion::parse("1.0.0-alpha").unwrap(),
            ChocoVersion::parse("1.0.0-rc").unwrap(),
            ChocoVersion::parse("1.2.0-55").unwrap(),
            ChocoVersion::parse("1.2").unwrap(),
            ChocoVersion::parse("5.0-beta.55").unwrap(),
            ChocoVersion::parse("5.0-beta.56").unwrap(),
            ChocoVersion::parse("6.2.0").unwrap(),
        ];

        versions.sort();

        assert_eq!(versions, expected);
    }
}
