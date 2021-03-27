// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Section responsible for allowing requests to be sent to remote locations.

use reqwest::blocking::Client;
use reqwest::{header, Url};

use crate::response::HtmlResponse;

/// The name of the application + the version, which should be sent with every
/// request to the websites.
static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

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
        let client = Client::builder()
            .user_agent(APP_USER_AGENT)
            .default_headers(headers!(
                header::ACCEPT_LANGUAGE => "en-US, en;q=0.8, *;q=0.5",
                header::DNT => "1",
                header::UPGRADE_INSECURE_REQUESTS => "1"
            ))
            .build()
            .unwrap();

        WebRequest { client }
    }

    /// Makes a request to a website and requesting the html at the location
    /// without downloading the actual upstream content. THe function also
    /// verifies that the returned response have the mime type set to
    /// `text/html`, otherwise an error is returned.
    ///
    /// The `Ok` value should be an instance of [HtmlResponse], and the links in
    /// the response can be found by calling the
    /// [read](crate::response::HtmlResponse::read) function.
    pub fn get_html_response(&self, url: &str) -> Result<HtmlResponse, Box<dyn std::error::Error>> {
        let url = Url::parse(url)?;

        let client = &self.client;

        let response = client
            .get(url)
            .header(header::ACCEPT, "text/html;charset=UTF-8")
            .send()?;

        Ok(HtmlResponse::new(response))
    }
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
    fn get_html_response_should_set_404_status_code() {
        let request = WebRequest::create();

        let response = request
            .get_html_response("https://httpbin.org/status/404")
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
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
}
