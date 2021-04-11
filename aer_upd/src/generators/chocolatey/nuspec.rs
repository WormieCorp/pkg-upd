// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Code related to creating Chocolatey metadata files (nuspec).

use std::fs;
use std::path::Path;

use aer_data::prelude::chocolatey::*;
use aer_data::prelude::*;
use xml::writer::XmlEvent;
use xml::EmitterConfig;

use crate::generators::PackageGenerator;

/// Generates the xml nuspec file based on the information specified in
/// [ChocolateyMetadata].
///
/// It will automatically generate a file directive for a local tools directory.
impl PackageGenerator for ChocolateyMetadata {
    fn generate(&self, work_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        if self.id().trim().is_empty() {
            return Err(std::io::Error::from(std::io::ErrorKind::NotFound).into());
        }

        let work_dir = work_dir.join(self.id());

        if !work_dir.exists() {
            fs::create_dir_all(&work_dir)?;
        } else {
            for entry in fs::read_dir(&work_dir)? {
                let path = entry?.path();
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
        }

        create_nuspec_file(work_dir, self)
    }
}

const XML_TEST_COMMENT: &str = "\
Do not remove this test for UTF-8: If “Ω” doesn't appear as greek uppercase omega letter
enclosed in quotation marks, you should use an editor that supports UTF-8, not this one.";

macro_rules! write_element {
    ($w:expr, $name:literal, $value:expr) => {
        $w.write(::xml::writer::XmlEvent::start_element($name))?;
        $w.write(::xml::writer::XmlEvent::characters(&$value.trim()))?;
        $w.write(::xml::writer::XmlEvent::end_element())?;
    };
    ($w:expr, $name:literal, $($attr_name:literal => $attr_val:expr)+) => {
        $w.write(::xml::writer::XmlEvent::start_element($name)$(.attr($attr_name, &$attr_val.trim()))*)?;
        $w.write(::xml::writer::XmlEvent::end_element())?;
    };
}

macro_rules! write_element_cdata {
    ($w:expr, $name:literal, $value:expr) => {
        $w.write(::xml::writer::XmlEvent::start_element($name))?;
        $w.write(::xml::writer::XmlEvent::cdata(&$value.trim()))?;
        $w.write(::xml::writer::XmlEvent::end_element())?;
    };
}

macro_rules! write_element_option {
    ($w:expr, $name:literal,$value:expr) => {
        if let Some(ref val) = $value {
            write_element!($w, $name, val.as_str());
        }
    };
}

fn create_nuspec_file<P: AsRef<Path>>(
    work_dir: P,
    data: &ChocolateyMetadata,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path = work_dir.as_ref().join(format!("{}.nuspec", data.id()));
    let mut file = fs::File::create(file_path)?;
    let mut writer = EmitterConfig::new()
        .perform_indent(true)
        .create_writer(&mut file);

    {
        let package_event = XmlEvent::start_element("package")
            .default_ns("http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd");
        writer.write(package_event)?;
    }

    writer.write(XmlEvent::comment(XML_TEST_COMMENT))?;
    writer.write(XmlEvent::start_element("metadata"))?;

    write_element!(writer, "id", data.id());
    {
        let version = data.version.to_choco();
        write_element!(writer, "version", version.to_string());
    }
    write_element_option!(writer, "packageSourceUrl", data.package_source_url);
    write_element!(writer, "owners", data.maintainers().join(","));
    write_element_option!(writer, "title", data.title);
    write_element!(writer, "authors", data.authors().join(","));
    write_element_option!(writer, "projectUrl", data.project_url);
    write_element_option!(writer, "iconUrl", data.icon_url);
    write_element_option!(writer, "copyright", data.copyright);
    if let Some(ref url) = data.license_url {
        write_element!(writer, "licenseUrl", url.as_str());
        write_element!(
            writer,
            "requireLicenseAcceptance",
            data.require_license_acceptance.to_string()
        );
    }

    write_element_option!(writer, "projectSourceUrl", data.project_source_url);
    write_element_option!(writer, "docsUrl", data.documentation_url);
    write_element_option!(writer, "mailingListUrl", data.mailing_list_url);
    write_element_option!(writer, "bugTrackerUrl", data.issues_url);
    write_element!(writer, "tags", data.tags().join(" "));
    write_element_option!(writer, "summary", data.summary);

    if let Description::Text(ref text) = data.description() {
        write_element_cdata!(writer, "description", text);
    }
    // Not decided if description should be loaded at this point, or before calling
    // the generate function

    write_element_option!(writer, "releaseNotes", data.release_notes);

    if !data.dependencies().is_empty() {
        writer.write(XmlEvent::start_element("dependencies"))?;
        for (id, version) in data.dependencies() {
            if let Some(version) = version {
                write_element!(writer,
                "dependency",
                "id" => id
                "version" => &version.to_string());
            } else {
                write_element!(writer, "dependency", "id" => id);
            }
        }
        writer.write(XmlEvent::end_element())?; // End of <dependencies>
    }

    writer.write(XmlEvent::end_element())?; // End of <metadata>

    writer.write(XmlEvent::start_element("files"))?;
    const DEFAULT_FILE: &str = if cfg!(windows) {
        "tools\\**"
    } else {
        "tools/**"
    };

    write_element!(writer,
        "file",
        "src" => DEFAULT_FILE
        "target" => "tools");
    for (src, target) in data.files() {
        let src = if cfg!(windows) {
            src.to_string_lossy().replace("/", "\\")
        } else {
            src.to_string_lossy().replace("\\", "/")
        };

        if src != DEFAULT_FILE {
            write_element!(writer, "file", "src" => src "target" => target);
        }
    }

    writer.write(XmlEvent::end_element())?; // End of <files>
    writer.write(XmlEvent::end_element())?; // End of <package>

    Ok(())
}

#[cfg(test)]
mod tests {
    use aer_data::prelude::*;
    use assert_fs::fixture::TempDir;
    use assert_fs::prelude::*;
    use predicates::prelude::*;
    use rstest::rstest;

    use super::*;

    mod metadata {
        use super::*;

        #[test]
        fn generate_should_create_expected_work_directory() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test")
                .assert(predicate::path::exists().and(predicates::path::is_dir()));
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_clean_existing_files() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);
            let input_file = temp.child("test/non.nuspec");
            input_file.touch().unwrap();

            meta.generate(temp.path()).unwrap();

            input_file.assert(predicate::path::missing());
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_clean_existing_directories() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);
            let input_dir = temp.child("test/tools");
            input_dir.create_dir_all().unwrap();

            meta.generate(temp.path()).unwrap();

            input_dir.assert(predicate::path::missing());
            temp.close().unwrap();
        }

        #[test]
        #[should_panic = "Kind(NotFound)"]
        fn generate_should_panic_if_identifier_is_empty() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::default();

            let r = meta.generate(temp.path());
            temp.close().unwrap();
            r.unwrap();
        }

        #[test]
        fn generate_should_create_nuspec_file() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test-package", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test-package/test-package.nuspec")
                .assert(predicate::path::exists().and(predicate::path::is_file()));
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_package_element() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test-package", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test-package/test-package.nuspec").assert(
                    predicate::path::exists().and(
                        predicate::str::contains(r#"<package xmlns="http://schemas.microsoft.com/packaging/2015/06/nuspec.xsd">"#).and(
                            predicate::str::contains("</package>"))
                        .from_utf8().from_file_path()));
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_test_comment() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::path::exists().and(
                    predicate::str::contains(format!("<!-- {} -->", XML_TEST_COMMENT))
                        .from_utf8()
                        .from_file_path(),
                ),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_metadata_element() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::path::exists().and(
                    predicate::str::contains("<metadata>")
                        .and(predicate::str::contains("</metadata>"))
                        .from_utf8()
                        .from_file_path(),
                ),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_identifier_element() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("awesome-souce", true);

            meta.generate(temp.path()).unwrap();

            temp.child("awesome-souce/awesome-souce.nuspec").assert(
                predicate::path::exists().and(
                    predicate::str::contains("<id>awesome-souce</id>")
                        .from_utf8()
                        .from_file_path(),
                ),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_version_element() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.version = SemVersion::parse("5.2.1-alpha.66+99").unwrap().into();

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<version>5.2.1-alpha0066")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_package_source_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.package_source_url =
                Some(Url::parse("https://github.com/AdmiringWorm/chocolatey-packages").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<packageSourceUrl>https://github.com/AdmiringWorm/chocolatey-packages</packageSourceUrl>")
                .from_utf8()
                .from_file_path()
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_package_source_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<packageSourceUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_owners() {
            let temp = TempDir::new().unwrap();
            let mut pkg = PackageMetadata::new("test");
            pkg.set_maintainers(&["AdmiringWorm", "gep13"]);
            let mut meta = ChocolateyMetadata::default();
            meta.update_from(pkg);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<owners>AdmiringWorm,gep13</owners>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_title() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_title("Test Package");

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<title>Test Package</title>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_title() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<title>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_authors() {
            let temp = TempDir::new().unwrap();
            let pkg = PackageMetadata::new("test");
            let mut meta = ChocolateyMetadata::with_authors(&[
                "Vivek Élodie Michaelson",
                "Gostislav Wigburg Asìs",
                "Ørjan Petterson",
            ]);
            meta.update_from(pkg);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains(
                    "<authors>Vivek Élodie Michaelson,Gostislav Wigburg Asìs,Ørjan \
                     Petterson</authors>",
                )
                .from_utf8()
                .from_file_path(),
            );
            temp.close().unwrap();
        }
        #[test]
        fn generate_should_create_project_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.project_url = Some(Url::parse("https://test.com").unwrap());
            meta.generate(temp.path()).unwrap();
            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<projectUrl>https://test.com/</projectUrl>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }
        #[test]
        fn generate_should_not_create_project_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);
            meta.generate(temp.path()).unwrap();
            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<projectUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_icon_url() {
            let temp = TempDir::new().unwrap();
            let mut pkg = PackageMetadata::new("test");
            pkg.set_icon_url("https://example.ci/icon.png").unwrap();
            let mut meta = ChocolateyMetadata::default();
            meta.update_from(pkg);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<iconUrl>https://example.ci/icon.png</iconUrl>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_icon_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<iconUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }
        #[test]
        fn generate_should_create_copyright() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_copyright("Awesome copyright");

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<copyright>Awesome copyright</copyright>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_copyright() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<copyright>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_license_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.license_url = Some(Url::parse("https://chocolatey.com/license").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<licenseUrl>https://chocolatey.com/license</licenseUrl>")
                    .and(predicate::str::contains(
                        "<requireLicenseAcceptance>true</requireLicenseAcceptance>",
                    ))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_require_license_acceptance_to_false() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.license_url = Some(Url::parse("https://chocolatey.org").unwrap());
            meta.require_license_acceptance = false;

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<licenseUrl>https://chocolatey.org/</licenseUrl>")
                    .and(predicate::str::contains(
                        "<requireLicenseAcceptance>false</requireLicenseAcceptance>",
                    ))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[rstest(val, case(true), case(false))]
        fn generate_should_not_create_license_url(val: bool) {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.require_license_acceptance = val;

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<licenseUrl>")
                    .or(predicate::str::contains("<requireLicenseAcceptance>"))
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_project_source_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.project_source_url =
                Some(Url::parse("https://www.example.org/source-code").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains(
                    "<projectSourceUrl>https://www.example.org/source-code</projectSourceUrl>",
                )
                .from_utf8()
                .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_project_source_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<projectSourceUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_documentation_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_title("Test Package");
            meta.documentation_url = Some(Url::parse("https://example.com/docs").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<docsUrl>https://example.com/docs</docsUrl>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_documentation_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<docsUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_mailing_list_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.mailing_list_url = Some(Url::parse("https://github.com/mailing-list").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains(
                    "<mailingListUrl>https://github.com/mailing-list</mailingListUrl>",
                )
                .from_utf8()
                .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_mailing_list_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<mailingListUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_issues_url() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.issues_url = Some(Url::parse("https://github.com/WormieCorp/aer/issues").unwrap());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains(
                    "<bugTrackerUrl>https://github.com/WormieCorp/aer/issues</bugTrackerUrl>",
                )
                .from_utf8()
                .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_issues_url() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<bugTrackerUrl>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_issues_tags() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.add_tag("cli");
            meta.add_tag("awesome");

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<tags>test cli awesome</tags>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_issues_summary() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.summary = Some("Awesomeness overloaded".into());

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<summary>Awesomeness overloaded</summary>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_issues_summary() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<summary>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_description() {
            let text =
                "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor \
                 incididunt ut labore et dolore magna aliqua. Leo a diam sollicitudin tempor id \
                 eu nisl. Vitae elementum curabitur vitae nunc sed velit dignissim sodales. \
                 Dictum non consectetur a erat. Vel risus commodo viverra maecenas accumsan. \
                 Ultricies mi eget mauris pharetra et. Pellentesque habitant morbi tristique \
                 senectus et netus et malesuada. Cras semper auctor neque vitae. Sit amet \
                 consectetur adipiscing elit duis tristique sollicitudin. Mauris commodo quis \
                 imperdiet massa tincidunt. Viverra aliquet eget sit amet tellus \
                 cras.\n\nMolestie a iaculis at erat pellentesque adipiscing commodo elit at. In \
                 nulla posuere sollicitudin aliquam ultrices sagittis orci a scelerisque. Erat \
                 nam at lectus urna duis. Leo in vitae turpis massa sed elementum. Iaculis urna \
                 id volutpat lacus. Nisl nunc mi ipsum faucibus. Eu augue ut lectus arcu \
                 bibendum. Senectus et netus et malesuada. Egestas maecenas pharetra convallis \
                 posuere morbi leo urna molestie. Aenean et tortor at risus viverra adipiscing at \
                 in tellus. Blandit volutpat maecenas volutpat blandit aliquam etiam erat velit \
                 scelerisque. Fames ac turpis egestas sed tempus urna et pharetra. Proin libero \
                 nunc consequat interdum varius sit amet mattis. Id faucibus nisl tincidunt eget \
                 nullam non nisi est. Lacinia quis vel eros donec ac odio tempor orci dapibus. Mi \
                 in nulla posuere sollicitudin aliquam ultrices sagittis. Risus nullam eget felis \
                 eget nunc.\n\nSit amet aliquam id diam maecenas ultricies mi eget mauris. \
                 Consectetur purus ut faucibus pulvinar. Sit amet tellus cras adipiscing enim. \
                 Platea dictumst vestibulum rhoncus est pellentesque. Facilisis leo vel fringilla \
                 est ullamcorper eget nulla facilisi. Risus viverra adipiscing at in tellus \
                 integer feugiat scelerisque. Vestibulum lorem sed risus ultricies. Aliquam etiam \
                 erat velit scelerisque in. Netus et malesuada fames ac turpis egestas integer. \
                 Maecenas sed enim ut sem viverra.";
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_description_str(text);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains(format!(
                    "<description><![CDATA[{}]]></description>",
                    text
                ))
                .from_utf8()
                .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_description() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<description>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_issues_release_notes() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_release_notes("random release notes");

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<releaseNotes>random release notes</releaseNotes>")
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_issues_release_notes() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<releaseNotes>")
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_issues_dependencies() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_dependencies(&[("chocolatey-core.extension", "2.1.0"), ("python3", "")]);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<dependencies>")
                    .and(predicate::str::contains(
                        r#"<dependency id="chocolatey-core.extension" version="2.1.0" />"#,
                    ))
                    .and(predicate::str::contains(r#"<dependency id="python3" />"#))
                    .and(predicate::str::contains("</dependencies>"))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_issues_dependencies() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<dependencies>")
                    .or(predicate::str::contains("<dependency"))
                    .or(predicate::str::contains("<dependencies />"))
                    .not()
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_always_files() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("test", true);

            meta.generate(temp.path()).unwrap();
            let sep = if cfg!(windows) { '\\' } else { '/' };

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<files>")
                    .and(predicate::str::contains(format!(
                        r#"<file src="tools{}**" target="tools" />"#,
                        sep
                    )))
                    .and(predicate::str::contains("</files>"))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_not_create_duplicate_tools_source() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.set_files(&[("tools/**", "tools")]);

            meta.generate(temp.path()).unwrap();
            let sep = if cfg!(windows) { '\\' } else { '/' };

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<files>")
                    .and(
                        predicate::str::contains(format!(
                            r#"<file src="tools{}**" target="tools" />"#,
                            sep
                        ))
                        .count(1),
                    )
                    .and(predicate::str::contains("</files>"))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_custom_files() {
            let temp = TempDir::new().unwrap();
            let mut meta = ChocolateyMetadata::with_id("test", true);
            meta.add_file("legal/**", "legal");
            meta.add_file("test\\**", "test");

            meta.generate(temp.path()).unwrap();
            let sep = if cfg!(windows) { '\\' } else { '/' };

            temp.child("test/test.nuspec").assert(
                predicate::str::contains("<files>")
                    .and(predicate::str::contains(format!(
                        r#"<file src="tools{}**" target="tools" />"#,
                        sep
                    )))
                    .and(predicate::str::contains(format!(
                        r#"<file src="legal{}**" target="legal" />"#,
                        sep
                    )))
                    .and(predicate::str::contains(format!(
                        r#"<file src="test{}**" target="test" />"#,
                        sep
                    )))
                    .and(predicate::str::contains("</files>"))
                    .from_utf8()
                    .from_file_path(),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_expected_empty_nuspec() {
            let temp = TempDir::new().unwrap();
            let meta = ChocolateyMetadata::with_id("authy-desktop", true);

            meta.generate(temp.path()).unwrap();
            let sep = if cfg!(windows) { '\\' } else { '/' };

            temp.child("authy-desktop/authy-desktop.nuspec").assert(
                predicate::path::exists()
                    .and(predicate::path::is_file())
                    .and(
                        predicate::str::similar(format!(
                            include_str!("../../../test-data/nuspecs/empty.nuspec"),
                            sep = sep
                        ))
                        .from_utf8()
                        .from_file_path(),
                    ),
            );
            temp.close().unwrap();
        }

        #[test]
        fn generate_should_create_expected_full_nuspec() {
            let temp = TempDir::new().unwrap();
            let mut pkg = PackageMetadata::new("InkScape");
            pkg.set_icon_url("https://cdn.jsdelivr.net/gh/chocolatey-community/chocolatey-coreteampackages@84a3a84e256daa3255c4a896eefbf8f5589fb842/icons/InkScape.svg").unwrap();
            pkg.set_license("https://git.launchpad.net/inkscape/tree/COPYING");
            pkg.set_maintainers(&["chocolatey-community"]);
            pkg.set_package_source_url("https://github.com/chocolatey-community/chocolatey-coreteampackages/tree/master/automatic/inkscape").unwrap();
            pkg.set_project_source_url("https://git.launchpad.net/inkscape/tree/")
                .unwrap();
            pkg.set_project_url("https://inkscape.org/");
            pkg.summary = "An Open Source vector graphics editor, with capabilities similar to \
                           Illustrator, CorelDraw, or Xara X, using the W3C standard Scalable \
                           Vector Graphics (SVG) file format."
                .into();
            let mut choco = ChocolateyMetadata::with_authors(&["Inkscape developers"]);
            choco.add_dependencies("chocolatey-core.extension", "1.3.3");
            choco.add_file("legal\\**", "legal");
            choco.add_file("tools\\**", "tools");
            choco.set_copyright("inkscape.org");
            choco.set_description_str("Inkscape is an open-source vector graphics editor similar to Adobe Illustrator, Corel Draw, Freehand, or Xara X. What sets Inkscape apart is its use of Scalable Vector Graphics (SVG), an open XML-based W3C standard, as the native format.

Inkscape supports many advanced SVG features (markers, clones, alpha blending, etc.) and great care is taken in designing a streamlined interface. It is very easy to edit nodes, perform complex path operations, trace bitmaps and much more. We also aim to maintain a thriving user and developer community by using open, community-oriented development.

All Inkscape projects may be exported in formats friendly to web browsers or commercial printer rooms. It is cross-platform, which means it is easy to run on Windows, Mac OS X, and Linux distributions. Visit the Download page to install or share this application now.

![InkScape](https://i.imgur.com/hvdwGBt.png)

[More screenshots](https://inkscape.org/en/about/screenshots/).

## Features

* Object creation: drawing, shape tools, text tool, bitmaps, clones
* Object manipulation: transformations, z-order operations, grouping, layers, aligment
* Fill and stroke: color selector, color picker tool, copy/paste style, pattern fills, dashed strokes, with many predefined dash patterns, path markers (ending, middle and/or beginning marks, e.g. arrowheads)
* Operations on paths
* Rendering: fully anti-aliased display, alpha transparency support for display and PNG export
* File formats: SVG, PNG, OpenDocument Drawing, DXF, sk1, PDF, EPS and PostScript export formats and more
* Command line options for export and conversions");
            choco.set_release_notes("https://inkscape.org/release/inkscape-1.0.2/#left-column");
            choco.set_title("Inkscape");
            choco.set_tags(&[
                "editor",
                "foss",
                "cross-platform",
                "svg",
                "vector-graphics",
                "icons",
                "graphics",
                "export",
                "drawing",
                "art",
                "admin",
            ]);
            choco.documentation_url = Some(Url::parse("https://inkscape.org/en/learn/").unwrap());
            choco.issues_url = Some(Url::parse("https://bugs.launchpad.net/inkscape").unwrap());
            choco.mailing_list_url =
                Some(Url::parse("https://inkscape.org/en/community/mailing-lists/").unwrap());
            choco.require_license_acceptance = false;
            choco.version = Versions::parse("1.0.2").unwrap();
            choco.update_from(&pkg);

            choco.generate(temp.path()).unwrap();

            let sep = if cfg!(windows) { '\\' } else { '/' };

            temp.child("inkscape/inkscape.nuspec").assert(
                predicate::path::exists()
                    .and(predicate::path::is_file())
                    .and(
                        predicate::str::similar(format!(
                            include_str!("../../../test-data/nuspecs/full.nuspec"),
                            sep = sep
                        ))
                        .from_utf8()
                        .from_file_path(),
                    ),
            );
            temp.close().unwrap();
        }
    }
}
