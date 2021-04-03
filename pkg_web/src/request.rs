// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Section responsible for allowing requests to be sent to remote locations.

use std::collections::HashMap;

use lazy_static::lazy_static;
use log::info;
use reqwest::blocking::{Client, Response};
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::{header, StatusCode, Url};

use crate::errors::WebError;
use crate::response::{BinaryResponse, HtmlResponse, ResponseType};

/// The name of the application + the version, which should be sent with every
/// request to the websites.
const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

lazy_static! {
    static ref ACCEPTED_TYPES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("html", "text/html; charset=UTF-8");
        map.insert("binary", "application/octet-stream");

        map
    };
}

/// Holds the necessary information to create requests to websites.
/// Also responsible for having a structure instance that can be used to get
/// different types of responses.
///
/// ## Examples
///
/// Aquiring html response.
/// ```
/// use pkg_web::WebRequest;
///
/// let request = WebRequest::create();
/// let response = request
///     .get_html_response("https://httpbin.org/get")
///     .unwrap();
/// ```
pub struct WebRequest {
    client: Client,
}

macro_rules! headers {
    ($($key:expr=>$value:literal),+) => {
        {
            let mut headers = ::reqwest::header::HeaderMap::new();
            $(headers.insert($key, ::reqwest::header::HeaderValue::from_static($value));)*

            headers
        }
    };
}

impl WebRequest {
    /// Creates a new instance of a web request. This also creates a client with
    /// the information set to the current application+version, a do not track
    /// header and a header requesting to upgrade insecure requests.
    pub fn create() -> WebRequest {
        let mut client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers!(
                header::ACCEPT_LANGUAGE => "en-US, en;q=0.8, *;q=0.5",
                header::DNT => "1",
                header::UPGRADE_INSECURE_REQUESTS => "1"
            ));
        if cfg!(windows) {
            client = client.use_rustls_tls();
        }

        WebRequest {
            client: client.build().unwrap(),
        }
    }

    /// Makes a request to a website and requesting the html at the location
    /// without downloading the actual upstream content.
    ///
    /// The `Ok` value should be an instance of [HtmlResponse], and the links in
    /// the response can be found by calling the
    /// [read](crate::response::HtmlResponse::read) function.
    pub fn get_html_response(&self, url: &str) -> Result<HtmlResponse, WebError> {
        let url = Url::parse(url).map_err(|err| WebError::Other(err.to_string()))?;

        let client = &self.client;

        let response = client
            .get(url)
            .header(header::ACCEPT, ACCEPTED_TYPES["html"])
            .send()
            .map_err(WebError::Request)?;

        handle_exit_code(response, HtmlResponse::new)
    }

    /// Makes a request to a web endpoint and requests a result in the type of a
    /// binary without downloading the actual upstream content. If an etag
    /// or last_modified argument is specified, these will be sent along with
    /// the request and will return a [ResponseType::Updated] if the server
    /// responds with a not modified response, otherwise a wrapped binary
    /// response is returned that can be used to download the remote file.
    ///
    /// ## Arguments
    ///
    /// - `url`: The url to the binary file that should possibly be downloaded.
    /// - `etag`: The etag that was previously returned by the server, will be
    ///   used to check if the binary file have changed.
    /// - `last_modified`: A string with the information of when the binary file
    ///   was last modified, this usually is a response previously sent my the
    ///   server.
    ///
    /// ## Notes
    ///
    /// - _Remember to set the work directory before doing the actual
    ///   downloading_
    pub fn get_binary_response(
        &self,
        url: &str,
        etag: Option<&str>,
        last_modified: Option<&str>,
    ) -> Result<ResponseType<BinaryResponse>, WebError> {
        let url = Url::parse(url).map_err(|err| WebError::Other(err.to_string()))?;

        let client = &self.client;
        let headers = {
            let mut headers = HeaderMap::new();
            headers.insert(
                header::ACCEPT,
                HeaderValue::from_static(ACCEPTED_TYPES["binary"]),
            );
            if let Some(etag) = etag {
                let new_etag = format!("\"{}\"", etag.trim_matches('"'));

                headers.insert(
                    header::IF_NONE_MATCH,
                    HeaderValue::from_str(&new_etag)
                        .map_err(|err| WebError::Other(err.to_string()))?,
                );
            }
            if let Some(last_modified) = last_modified {
                headers.insert(
                    header::IF_MODIFIED_SINCE,
                    HeaderValue::from_str(last_modified)
                        .map_err(|err| WebError::Other(err.to_string()))?,
                );
            }

            headers
        };

        let response = client
            .get(url.clone())
            .headers(headers)
            .send()
            .map_err(WebError::Request)?;
        let status = response.status();

        if status == StatusCode::NOT_MODIFIED {
            info!("The web server responded with status: {}!", status);

            Ok(ResponseType::Updated(status.as_u16()))
        } else {
            handle_exit_code(response, move |rsp| {
                ResponseType::New(BinaryResponse::new(rsp, url), status.as_u16())
            })
        }
    }
}

fn handle_exit_code<T, F: FnOnce(Response) -> T>(
    response: Response,
    creation: F,
) -> Result<T, WebError> {
    if !response.status().is_success() {
        return match response.error_for_status() {
            Err(err) => Err(WebError::Request(err)),
            Ok(_) => unreachable!(),
        };
    }

    info!(
        "The web server responded with status: {}!",
        response.status()
    );

    Ok(creation(response))
}

#[cfg(test)]
mod tests {
    use reqwest::StatusCode;

    use super::*;
    use crate::response::*;

    #[test]
    fn create_should_build_client_with_expected_values() {
        let _ = WebRequest::create();

        // Nothing more is done, as we only test if a panic happens which we do
        // not expect.
    }

    #[test]
    fn get_html_response_should_create_response() {
        let url = Url::parse("https://httpbin.org/get").unwrap();
        let request = WebRequest::create();

        let response = request.get_html_response(url.as_str()).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.response().url(), &url);
    }

    #[test]
    #[should_panic(expected = "Status(404)")]
    fn get_html_response_should_give_error_on_404_status_code() {
        let request = WebRequest::create();

        let _ = request
            .get_html_response("https://httpbin.org/status/404")
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Status(500)")]
    fn get_html_response_should_give_error_on_error_response() {
        let request = WebRequest::create();

        let _ = request
            .get_html_response("https://httpbin.org/status/500")
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Status(404)")]
    fn get_binary_response_should_give_error_on_404_status_code() {
        let request = WebRequest::create();

        let _ = request
            .get_binary_response("https://httpbin.org/status/404", None, None)
            .unwrap();
    }

    #[test]
    #[should_panic(expected = "Status(500)")]
    fn get_binary_response_should_give_error_on_error_response() {
        let request = WebRequest::create();

        let _ = request
            .get_binary_response("https://httpbin.org/status/500", None, None)
            .unwrap();
    }

    #[test]
    fn get_html_response_should_follow_redirection() {
        let final_url =
            Url::parse("https://github.com/WormieCorp/Faker.NET.Portable/releases/tag/2.6.0")
                .unwrap();
        let url = "https://github.com/WormieCorp/Faker.NET.Portable/releases/latest";
        let request = WebRequest::create();

        let response = request.get_html_response(url).unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.response().url(), &final_url);
    }

    #[test]
    #[cfg_attr(
        not(windows),
        should_panic(expected = "failed to lookup address information")
    )]
    #[cfg_attr(windows, should_panic(expected = "No such host is known."))]
    fn get_html_response_should_be_error_on_non_existing_urls() {
        let request = WebRequest::create();

        request
            .get_html_response("https://chocolatyyy.org")
            .unwrap();
    }

    #[test]
    fn get_binary_response_should_return_already_updated_response_by_etag() {
        let request = WebRequest::create();
        let response = request.get_binary_response("https://github.com/codecov/codecov-exe/releases/download/1.13.0/codecov-linux-x64.zip", Some("\"e3d41332a09dd059961efade340c12da\""), None).unwrap();

        assert_eq!(response, ResponseType::Updated(304));
    }

    #[test]
    fn get_binary_response_should_return_already_updated_response_by_last_modified() {
        let request = WebRequest::create();
        let response = request.get_binary_response("https://github.com/codecov/codecov-exe/releases/download/1.13.0/codecov-linux-x64.zip", None, Some("Tue, 16 Feb 2021 03:33:36 GMT")).unwrap();

        assert_eq!(response, ResponseType::Updated(304));
    }
}
