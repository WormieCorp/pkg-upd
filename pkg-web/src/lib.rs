// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! This crate allows requesting different kind of websites remotely, as well as
//! downloading binary files and extracting link items.
//!
//! ## Examples
//!
//! Aquiring the links from an html page, and asserting that 4 links was
//! returned!
//!
//! ```
//! use pkg_web::*;
//!
//! let request = WebRequest::create();
//! let response = request
//!     .get_html_response("https://httpbin.org/links/5/2")
//!     .unwrap();
//! let (parent_link, links) = response.read(None).unwrap();
//!
//! assert_eq!(links.len(), 4);
//! ```

mod elements;

pub mod request;
pub mod response;

pub use elements::{LinkElement, LinkType};
pub use request::WebRequest;
pub use response::WebResponse;
