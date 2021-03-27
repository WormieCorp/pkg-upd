// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use std::io::Read;
use std::path::Path;

use log::{debug, error};
use pkg_data::PackageData;

use crate::parsers::{errors, DataReader};

pub struct TomlParser;

/// Implements the trait necessary for reading files that are stored in the
/// `TOML` language.
/// See enhancement issue: #1
impl DataReader for TomlParser {
    fn can_handle_file(&self, path: &Path) -> bool {
        if let Some(path) = path.to_str() {
            path.ends_with(".pkg.toml")
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
                    let fmt = format!("{}", err);
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

    use pkg_data::prelude::*;
    use pkg_version::{SemVersion, Versions};
    use rstest::rstest;
    use url::Url;

    use super::*;

    struct ErrorReader {
        kind: ErrorKind,
    }

    impl Read for ErrorReader {
        fn read(&mut self, _: &mut [u8]) -> std::result::Result<usize, std::io::Error> {
            Err(Error::from(self.kind))
        }
    }

    #[rstest(
        file,
        case("test-package.toml"),
        case("test-package.pkg.yml"),
        case("test-package.xml")
    )]
    fn can_handle_file_returns_false_for_non_pkg_toml_files(file: &str) {
        let path = PathBuf::from_str(file).unwrap();
        let parser = TomlParser;

        let result = parser.can_handle_file(&path);

        assert!(!result);
    }

    #[rstest(
        kind,
        case(ErrorKind::NotFound),
        case(ErrorKind::PermissionDenied),
        case(ErrorKind::UnexpectedEof)
    )]
    fn read_data_should_error_on_io_access_failed(kind: ErrorKind) {
        let parser = TomlParser;
        let mut reader = ErrorReader { kind };

        let result = parser.read_data(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn read_data_should_error_on_wrong_data_format() {
        const VAL: &[u8] = b"This deserialization should fail!";
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;

        let result = parser.read_data(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn read_data_should_error_on_missing_required_value() {
        const VAL: &[u8] = br#"[metadata]
        id = "test-package"#;
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;

        let result = parser.read_data(&mut reader);

        assert!(result.is_err());
    }

    #[test]
    fn read_data_should_succeed_on_required_values_defined() {
        const VAL: &[u8] = include_bytes!("../../test-data/basic-metadata.toml");
        let mut reader = BufReader::new(VAL);
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

        let result = parser.read_data(&mut reader).unwrap();

        assert_eq!(result, expected);
    }

    #[test]
    fn read_data_should_accept_license_expression() {
        const VAL: &[u8] = include_bytes!("../../test-data/license-expression.toml");
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::Expression("MIT".to_owned()));

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_url() {
        const VAL: &[u8] = include_bytes!("../../test-data/license-url.toml");
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected.metadata_mut().set_license(LicenseType::Location(
            Url::parse("https://github.com/WormieCorp/pkg-upd/LICENSE.txt").unwrap(),
        ));

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_expression_and_url() {
        const VAL: &[u8] = include_bytes!("../../test-data/license-short.toml");
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::ExpressionAndLocation {
                url: Url::parse("https://github.com/WormieCorp/pkg-upd/LICENSE.txt").unwrap(),
                expression: "MIT".into(),
            });

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_license_in_seperate_section() {
        const VAL: &[u8] = include_bytes!("../../test-data/license-long.toml");
        let mut reader = BufReader::new(VAL);
        let parser = TomlParser;
        let mut expected = PackageData::new("test-package");
        expected
            .metadata_mut()
            .set_license(LicenseType::ExpressionAndLocation {
                url: Url::parse("https://github.com/WormieCorp/pkg-upd/LICENSE.txt").unwrap(),
                expression: "MIT".into(),
            });

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_accept_chocolatey_arguments() {
        const VAL: &[u8] = include_bytes!("../../test-data/metadata-choco.toml");
        let mut reader = BufReader::new(VAL);
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

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }

    #[test]
    fn read_data_should_deserialize_all_data() {
        const VAL: &[u8] = include_bytes!("../../test-data/deserialize-full.pkg.toml");
        let mut reader = BufReader::new(VAL);
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
                choco._type = ChocolateyUpdaterType::Archive;
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

        let actual = parser.read_data(&mut reader).unwrap();

        assert_eq!(actual, expected);
    }
}
