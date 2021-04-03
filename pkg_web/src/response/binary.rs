// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use std::fs::File;
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::{info, warn};
use reqwest::blocking::Response;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, Url};

use crate::response::WebError;
use crate::WebResponse;

/// Contains functions and items necessary for parsing and downloading binary
/// files.
///
/// Implements the [WebResponse] trait, and are not meant to be created directly
/// by a user.
#[derive(Debug)]
pub struct BinaryResponse {
    response: Response,
    url: Url,
    work_dir: PathBuf,
}

impl PartialEq for BinaryResponse {
    fn eq(&self, rhs: &BinaryResponse) -> bool {
        self.work_dir == rhs.work_dir // We do not compare the actual response, as it is not interesting
    }
}

impl BinaryResponse {
    /// Creates a new instance of the [BinaryResponse] structure to hold the
    /// current response, and allow downloading the remote file from the content
    /// response.
    pub fn new(response: Response, url: Url) -> BinaryResponse {
        BinaryResponse {
            response,
            url,
            work_dir: PathBuf::new(),
        }
    }

    /// Sets the current work directory (the directory where files will be
    /// downloaded). If this function is never called, the current directory
    /// (based on the execution location of the program) will be used. As such,
    /// it should always be used.
    pub fn set_work_dir(&mut self, path: &Path) {
        self.work_dir = PathBuf::from(path);
    }

    /// Tries to get the name of the remote file by either reading the
    /// disposition header, or checking the url if it contains an extension.
    pub fn file_name(&self) -> Option<String> {
        if let Some(name) = get_from_disposition(self.response.headers()) {
            Some(name)
        } else if let Some(name) = get_from_url(self.response.url()) {
            Some(name)
        } else {
            None
        }
    }
}

fn get_from_url(url: &Url) -> Option<String> {
    let segments = url.path_segments()?;
    let mut extension = String::new();

    for segment in segments {
        let path = PathBuf::from_str(segment);
        if path.is_err() {
            continue;
        }

        let path = path.unwrap();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if ext.is_empty() {
                continue;
            }
            if !extension.is_empty() {
                extension.clear()
            }
            extension.push_str(segment);
        }
    }

    if extension.is_empty() {
        None
    } else {
        Some(extension)
    }
}

fn get_from_disposition(headers: &HeaderMap<HeaderValue>) -> Option<String> {
    if let Some(disposition) = headers
        .get(header::CONTENT_DISPOSITION)
        .and_then(|d| d.to_str().ok())
    {
        if let Some(index) = disposition.find("filename") {
            let name: String = disposition
                .chars()
                .skip(index + 8)
                .skip_while(|c| *c != '=')
                .skip_while(|c| c.is_whitespace() || *c == '"' || *c == '=')
                .take_while(|c| *c != '"' && *c != ';')
                .collect();
            let name = name.trim().to_owned();

            if !name.is_empty() {
                return Some(name);
            }
        }
    }

    None
}

impl WebResponse for BinaryResponse {
    /// The path to a written file.
    type ResponseContent = PathBuf;

    fn response(&self) -> &Response {
        &self.response
    }

    /// Reads and downloads the response content.
    ///
    /// ## Arguments
    ///
    /// - `output`: The name of the file to create, if not specified it will be
    ///   resolved from the response.
    ///
    /// ## Returns
    ///
    /// On a successful download, the written path will be returned.
    ///
    /// ## Warning
    ///
    /// The `output` argument will be combined with the previously set work
    /// directory.
    fn read(self, output: Option<&str>) -> Result<Self::ResponseContent, WebError> {
        let output = if let Some(output) = output {
            output.into()
        } else {
            self.file_name()
                .ok_or_else(|| WebError::Other("Unable to extract file name request".into()))?
        };

        let output = self.work_dir.join(output);

        let mut response = self.response;

        info!("Downloading '{}' to '{}'", self.url, output.display());

        let file = File::create(output.clone()).map_err(WebError::IoError)?;
        let mut writer = BufWriter::new(&file);

        match response.copy_to(&mut writer) {
            Err(err) => {
                warn!("Failed to download '{}'", self.url);
                Err(WebError::Request(err))
            }
            Ok(_) => {
                info!("Successfully downloaded '{}'", output.display());
                Ok(output)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use reqwest::{header, Url};
    use rstest::rstest;

    use super::*;
    use crate::WebRequest;

    #[rstest(
        test,
        expected,
        case(
            "attachment; filename=Cake.Recipe.2.0.0.nupkg",
            "Cake.Recipe.2.0.0.nupkg"
        ),
        case("attachment; filename=\"Cake.nupkg\"", "Cake.nupkg"),
        case("attachment; filename=Test.exe; name=test", "Test.exe"),
        case("attachment; filename=  \"  Test.exe  \"  ; name=test", "Test.exe")
    )]
    fn get_from_disposition_should_get_file_name_from_disposition(
        test: &'static str,
        expected: &'static str,
    ) {
        let mut headers = HeaderMap::default();
        headers.append(
            header::CONTENT_DISPOSITION,
            header::HeaderValue::from_static(test),
        );

        let file_name = get_from_disposition(&headers);

        assert_eq!(file_name, Some(expected.into()));
    }

    #[test]
    fn get_from_disposition_should_be_none_when_no_disposition_in_header() {
        let headers = HeaderMap::new();

        let file_name = get_from_disposition(&headers);

        assert_eq!(file_name, None)
    }

    #[rstest(test, case("attachment"), case("inline; name=field-name"))]
    fn get_from_disposition_should_be_none_when_no_filename_in_disposition(test: &'static str) {
        let mut headers = HeaderMap::new();
        headers.append(
            header::CONTENT_DISPOSITION,
            header::HeaderValue::from_static(test),
        );

        let file_name = get_from_disposition(&headers);

        assert_eq!(file_name, None);
    }

    #[rstest(
        url,
        expected,
        case("https://eternallybored.org/misc/wget/1.21.1/32/wget.exe", "wget.exe"),
        case("https://github.com/clementine-player/Clementine/releases/download/1.3.1/ClementineSetup-1.3.1.exe", "ClementineSetup-1.3.1.exe"),
        case("https://sourceforge.net/projects/codeblocks/files/Binaries/20.03/Windows/codeblocks-20.03-setup.exe/download", "codeblocks-20.03-setup.exe")
    )]
    fn get_from_url_should_return_correct_file_name(url: &str, expected: &str) {
        let url = Url::parse(url).unwrap();

        let file_name = get_from_url(&url);

        assert_eq!(file_name, Some(expected.into()))
    }

    #[test]
    fn get_from_url_should_return_none_on_no_file_name() {
        let url = Url::parse("https://www.codeblocks.org/downloads/binaries/").unwrap();

        let file_name = get_from_url(&url);

        assert_eq!(file_name, None);
    }

    #[rstest(
        url,
        fname,
        case(
            "https://github.com/cake-build/cake/releases/download/v1.1.0/Cake-bin-coreclr-v1.1.0.zip",
            "Cake-bin-coreclr-v1.1.0.zip"
        ),
        case(
            "https://sourceforge.net/projects/codeblocks/files/Binaries/20.03/Windows/codeblocks-20.03-setup.exe/download",
             "codeblocks-20.03-setup.exe"
        )
    )]
    fn read_should_download_expected_file(url: &str, fname: &str) {
        let work_dir = std::env::temp_dir();
        let request = WebRequest::create();
        let mut response = request.get_binary_response(url, None, None).unwrap();
        response.set_work_dir(&work_dir);
        let expected = work_dir.join(fname);
        let path = response.read(None).unwrap();

        assert_eq!(path, expected.clone());

        let _ = std::fs::remove_file(expected);
    }
}
