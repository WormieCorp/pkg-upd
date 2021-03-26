// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Holds all supported types of response types, that can be used when creating
//! a package.

/// Contains code related to handling html responses.
mod html;

use std::collections::HashMap;

pub use html::HtmlResponse;
use lazy_static::lazy_static;
use reqwest::blocking::Response;
use reqwest::StatusCode;

lazy_static! {
    static ref MIME_TYPES: HashMap<&'static str, LinkType> = {
        let mut map = HashMap::new();
        map.insert("text/html", LinkType::Html);
        map.insert("text/plain", LinkType::Text);
        map.insert("text/json", LinkType::Json);
        map.insert("application/json", LinkType::Json);
        map.insert("text/css", LinkType::Css);
        map.insert("application/octet-stream", LinkType::Binary);
        map
    };
}

use crate::elements::LinkType;

/// Common trait to allow multiple response types to have the same functions to
/// be used.
///
/// ### See also
///
/// The following structures implements the [WebResponse] trait.
///
/// - [HtmlResponse](HtmlResponse): _Responsible of parsing html sites,
///   generally for aquiring links on a web page_.
pub trait WebResponse {
    /// The response content that will be returned by any implementation of
    /// [WebResponse]. This can be anything that would be expected by the
    /// response parser.
    type ResponseContent;

    /// Returns the actual response that was created by
    /// [WebRequest](crate::WebRequest).
    fn response(&self) -> &Response;

    /// Returns all of the headers that was returned by the web server.
    /// The headers can alternatively be gotten through the
    /// [response](WebResponse::response) function.
    fn get_headers(&self) -> HashMap<&str, &str> {
        let response = self.response();
        let mut headers = HashMap::with_capacity(response.headers().len());

        for (key, value) in response.headers() {
            if let Ok(val) = value.to_str() {
                headers.insert(key.as_str(), val);
            }
        }

        headers
    }

    /// Returns the status that was returned with the rest of the response.
    fn status(&self) -> StatusCode {
        self.response().status()
    }

    /// Reads the current response content, and if successful returns the a
    /// structure holding the necessary items found. This may return an
    /// error if the status code is a success code, or if the reading of the
    /// content failed.
    fn read(self, re: Option<&str>) -> Result<Self::ResponseContent, Box<dyn std::error::Error>>;
}

#[cfg(test)]
mod tests {
    use reqwest::blocking::get;

    use super::*;

    struct DummyResponse {
        response: Response,
    }

    impl DummyResponse {
        fn new(response: Response) -> DummyResponse {
            DummyResponse { response }
        }
    }

    impl WebResponse for DummyResponse {
        type ResponseContent = String;

        fn response(&self) -> &reqwest::blocking::Response {
            &self.response
        }

        fn read(
            self,
            _: Option<&str>,
        ) -> std::result::Result<
            <Self as WebResponse>::ResponseContent,
            std::boxed::Box<(dyn std::error::Error + 'static)>,
        > {
            unimplemented!()
        }
    }

    #[test]
    fn status_should_get_the_actual_status_code_of_response() {
        let response = get("https://httpbin.org/status/406").unwrap();

        let response = DummyResponse::new(response);

        assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
    }

    #[test]
    fn get_headers_should_get_the_actual_headers_for_the_response() {
        let response = get("https://httpbin.org/get").unwrap();

        let response = DummyResponse::new(response);

        let mut headers = response.get_headers();
        // let us remove the date and server header
        let _ = headers.remove("server");
        let _ = headers.remove("date");
        let _ = headers.remove("content-length"); // This can vary a little, so we remove it

        assert_eq!(headers, {
            let mut map = HashMap::new();
            map.insert("access-control-allow-origin", "*");
            map.insert("access-control-allow-credentials", "true");
            map.insert("content-type", "application/json");
            map.insert("connection", "keep-alive");

            map
        });
    }

    #[test]
    #[should_panic]
    fn just_for_coverage_on_test_dummy_structure() {
        let response = get("https://httpbin.org/get").unwrap();
        let response = DummyResponse::new(response);

        response.read(None).unwrap();
    }
}
