// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#![cfg_attr(docsrs, doc(cfg(feature = "toml_data")))]

use std::io::Read;
use std::path::Path;

use aer_data::PackageData;
use log::{debug, error};

use crate::parsers::{errors, DataReader};

pub struct TomlParser;

/// Implements the trait necessary for reading files that are stored in the
/// `TOML` language.
/// See enhancement issue: #1
impl DataReader for TomlParser {
    fn can_handle_file(&self, path: &Path) -> bool {
        if let Some(path) = path.to_str() {
            path.ends_with(".aer.toml")
        } else {
            false
        }
    }

    /// Reads and deserializes a `TOML` document in the specified reader passed
    /// to the function.
    fn read_data<T>(&self, reader: &mut T) -> Result<PackageData, errors::ParserError>
    where
        T: Read,
    {
        let config_data: PackageData = {
            let mut config_text = String::new();

            match reader.read_to_string(&mut config_text) {
                Err(err) => {
                    error!("Failed to read data: {:?}", err);
                    return Err(errors::ParserError::Loading(err));
                }
                Ok(size) => debug!("Read {} bytes!", size),
            }

            debug!("Deserializing TOML Package data");
            match toml::from_str(&config_text) {
                Err(err) => {
                    error!("Failed to deserialize package data: {:?}", err);
                    let fmt = err.to_string();
                    return Err(errors::ParserError::Deserialize(fmt));
                }
                Ok(data) => data,
            }
        };

        debug!("Package TOML data deserialized, returning package data!");

        Ok(config_data)
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Error, ErrorKind};
    use std::path::PathBuf;
    use std::str::FromStr;

    use aer_data::prelude::chocolatey::*;
    use aer_data::prelude::*;
    use rstest::rstest;

    use super::*;

    struct ErrorReader {
        kind: ErrorKind,
    }

    impl Read for ErrorReader {
        fn read(&mut self, _: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
            Err(Error::from(self.kind))
        }
    }

    #[rstest]
    #[case("test-package.toml")]
    #[case("test-package.aer.yml")]
    #[case("test-package.xml")]
    fn read_file_should_error_for_non_aer_toml_files(#[case] file: &str) {
        let path = PathBuf::from_str(file).unwrap();
        let parser = TomlParser;

        let r = parser.read_file(&path).unwrap_err();

        assert_eq!(
            r,
            errors::ParserError::Loading(Error::new(
                ErrorKind::InvalidData,
                format!("The file '{}' is not a supported type.", file)
            ))
        );
    }

    #[test]
    fn read_file_should_error_for_non_existing_file() {
        let path = PathBuf::from("test-file.aer.toml");
        let parser = TomlParser;

        let r = parser.read_file(&path).unwrap_err();

        assert_eq!(
            r,
            errors::ParserError::Loading(Error::new(
                ErrorKind::NotFound,
                format!("The file '{}' could not be found!", path.display())
            ))
        );
    }

    #[rstest]
    #[case(ErrorKind::NotFound)]
    #[case(ErrorKind::PermissionDenied)]
    #[case(ErrorKind::UnexpectedEof)]
    fn read_file_should_error_on_io_access_failed(#[case] kind: ErrorKind) {
        let parser = TomlParser;
        let mut reader = ErrorReader { kind };

        let r = parser.read_data(&mut reader).unwrap_err();

        assert_eq!(r, errors::ParserError::Loading(Error::from(kind)));
    }

    #[test]
    #[should_panic(expected = "expected an equals, found an identifier at line 1 column 6")]
    fn read_data_should_error_on_wrong_data_format() {
        const VAL: &[u8] = b"This deserialization should fail!";
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;

        let _ = parser.read_data(&mut reader).unwrap();
    }

    #[test]
    #[should_panic(expected = "missing field `summary` for key `metadata` at line 1 column 1")]
    fn read_data_should_error_on_missing_required_value() {
        const VAL: &[u8] = br#"[metadata]
        id = "test-package""#;
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;

        let _ = parser.read_data(&mut reader).unwrap();
    }

    #[test]
    fn read_data_should_succeed_on_required_values_defined() {
        let path = PathBuf::from("test-data/basic-metadata.aer.toml");
        let parser = TomlParser;
        let expected = {
            let mut pkg = PackageData::new("test-package");
            pkg.metadata_mut().set_license(LicenseType::None);
            pkg.metadata_mut().set_maintainers(&["AdmiringWorm"]);
            pkg.metadata_mut().set_project_url("https://test.com");
            pkg.metadata_mut().summary =
                "Some kind of summary (or description in some packages)".to_owned();
            pkg
        };

        let result = parser.read_file(&path).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn read_data_should_accept_license_expression() {
        let path = PathBuf::from("test-data/license-expression.aer.toml");
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::Expression("MIT".to_owned()));

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_url() {
        let path = PathBuf::from("test-data/license-url.aer.toml");
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected.metadata_mut().set_license(LicenseType::Location(
            Url::parse("https://github.com/WormieCorp/aer/LICENSE.txt").unwrap(),
        ));

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_expression_and_url() {
        let path = PathBuf::from("test-data/license-short.aer.toml");
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::ExpressionAndLocation {
                url: Url::parse("https://github.com/WormieCorp/aer/LICENSE.txt").unwrap(),
                expression: "MIT".into(),
            });

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_in_seperate_section() {
        let path = PathBuf::from("test-data/license-long.aer.toml");
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::ExpressionAndLocation {
                url: Url::parse("https://github.com/WormieCorp/aer/LICENSE.txt").unwrap(),
                expression: "MIT".into(),
            });

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_chocolatey_arguments() {
        let path = PathBuf::from("test-data/metadata-choco.aer.toml");
        let parser = TomlParser;
        let mut expected = {
            let mut pkg = PackageData::new("test-package");
            pkg.metadata_mut()
                .set_license(LicenseType::Expression("MIT".to_owned()));
            pkg.metadata_mut()
                .set_project_url("https:/_Software_Location_REMOVE_OR_FILL_OUT_");
            pkg
        };
        expected.metadata_mut().set_chocolatey({
            let mut choco = ChocolateyMetadata::with_authors(&["WormieCorp"]);
            choco.set_description_str("Some description");
            choco
        });

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_deserialize_all_data() {
        let path = PathBuf::from("test-data/deserialize-full.aer.toml");
        let parser = TomlParser;
        let expected = {
            let mut pkg = PackageData::new("astyle");
            let mut metadata = pkg.metadata_mut();
            metadata.set_license(LicenseType::ExpressionAndLocation {
                expression: "MIT".into(),
                url: Url::parse(
                    "https://sourceforge.net/p/astyle/code/HEAD/tree/trunk/AStyle/LICENSE.md",
                )
                .unwrap(),
            });
            metadata.set_maintainers(&["AdmiringWorm", "yying"]);
            metadata.set_project_url("http://astyle.sourceforge.net/");
            metadata.summary = "Artistic Style is a source code indenter, formater, and beutifier \
                                for the C, C++, C++/CLI, Objective-C, C# and Java programming \
                                languages."
                .into();
            metadata.set_chocolatey({
                let mut choco = ChocolateyMetadata::with_authors(&["Jim Pattee", "Tal Davidson"]);
                choco.set_description(Description::Location {
                    from: "./astyle.md".into(),
                    skip_start: 2,
                    skip_end: 1,
                });
                choco.version = Versions::SemVer(SemVersion::new(3, 1, 0));
                choco.set_title("Artistic Style");
                choco.set_copyright("Copyright (c) 2014 Jim Pattee, Tal Dividson");
                choco.require_license_acceptance = false;
                choco.documentation_url =
                    Some(Url::parse("http://astyle.sourceforge.net/astyle.html").unwrap());
                choco.issues_url =
                    Some(Url::parse("https://sourceforge.net/p/astyle/bugs").unwrap());
                choco.set_tags(&["astyle", "beautifier", "command-only", "development"]);
                choco.set_release_notes("[Software Changelog](http://astyle.sourceforge.net/notes.html)
[Package Changelog](https://github.com/AdmiringWorm/chocolatey-packages/blob/master/automatic/astyle/Changelog.md)");
                choco.add_dependencies("chocolatey-core.extension", "1.3.3");
                choco
            });

            pkg.updater_mut().set_chocolatey({
                let mut choco = ChocolateyUpdaterData::new();
                choco.embedded = true;
                choco.updater_type = ChocolateyUpdaterType::Archive;
                choco.parse_url = Some(ChocolateyParseUrl::UrlWithRegex {
                    url: Url::parse("https://sourceforge.net/projects/astyle/files/astyle/")
                        .unwrap(),
                    regex: r"astyle( |%20)(?P<version>[\d\.]+)/$".into(),
                });
                choco.add_regex("arch32", r"windows\.zip/download$");
                choco
            });

            pkg
        };

        let actual = parser.read_file(&path).unwrap();

        assert_eq!(actual, expected);
    }
}
