// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains the necessary logic for creating errors for the library.

use std::error::Error;
use std::fmt::Display;

/// Common error collector for different errors that can be found in the
/// library.
#[derive(Debug)]
pub enum WebError {
    /// An error happened when trying to request a web site.
    Request(reqwest::Error),
    /// An error that occurred while reading or writing to the file system
    IoError(std::io::Error),
    /// Any other type of error not covered by the other types.
    Other(String),
}

impl Error for WebError {}

impl Display for WebError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            WebError::Request(err) => err.fmt(f),
            WebError::IoError(err) => err.fmt(f),
            WebError::Other(val) => f.write_str(&val),
        }
    }
}
