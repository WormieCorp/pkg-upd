// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Holds all supported types of response types, that can be used when creating
//! a package.

/// Contains code related to handling binary responses (normally downloading).
mod binary;
/// Contains code related to handling html responses.
mod html;

use std::collections::HashMap;
use std::path::Path;

pub use binary::BinaryResponse;
pub use html::HtmlResponse;
use lazy_static::lazy_static;
use reqwest::blocking::Response;
use reqwest::StatusCode;

use crate::elements::LinkType;
use crate::errors::WebError;

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

/// A simple enumerator that holds information of wether the response returned
/// by a server said the content is up to date, or if there is new content
/// available.
///
/// ## Notes
///
/// - Calling any child response may panic if a function is called, and the
///   server returned an not modified response.
#[derive(Debug, PartialEq)]
pub enum ResponseType<T: WebResponse> {
    /// The response returned by the server was considered up to date, and no
    /// further processing is available. Sets the server status code as a
    /// member.
    Updated(u16),
    /// The response returned by the server is considered to be outdated and
    /// additional processing is necessary. Sets the type of the web
    /// response that can be used for further processing, and the status code
    /// returned by the server.
    New(T, u16),
}

/// Implements common functions that are also implemented on any child response.
impl<T: WebResponse> ResponseType<T> {
    /// Calls the read function on the underlying web response.
    ///
    /// ## Warning
    ///
    /// - Will panic if the response set is considered to be up to date.
    pub fn read(self, option: Option<&str>) -> Result<T::ResponseContent, WebError> {
        match self {
            ResponseType::Updated(status) => panic!(
                "Can not read an already updated response. Status Code: {}",
                status
            ),
            ResponseType::New(item, _) => item.read(option),
        }
    }
}

/// Implements functions that only makes sense to be called when the response
/// type is a binary response.
impl ResponseType<BinaryResponse> {
    /// Sets the directory that should be used when calling the child response.
    /// This function should not panic even if the response is considered up to
    /// date.
    pub fn set_work_dir(&mut self, path: &Path) {
        if let ResponseType::New(item, _) = self {
            item.set_work_dir(path)
        }
    }
}

/// Common trait to allow multiple response types to have the same functions to
/// be used.
///
/// ### See also
///
/// The following structures implements the [WebResponse] trait.
///
/// - [HtmlResponse](HtmlResponse): _Responsible of parsing html sites,
///   generally for aquiring links on a web page_.
/// - [BinaryResponse](BinaryResponse): _Responsible for downloading a remote
///   file to a specified location_
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
    fn read(self, re: Option<&str>) -> Result<Self::ResponseContent, WebError>;
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
        ) -> std::result::Result<<Self as WebResponse>::ResponseContent, WebError> {
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
