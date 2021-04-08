// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ParserError {
    NoParsers(PathBuf),
    Loading(std::io::Error),
    Deserialize(String),
    Other { inner: Box<dyn Error> },
}

impl fmt::Display for ParserError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParserError::Loading(err) => err.fmt(f),
            ParserError::Deserialize(s) => s.fmt(f),
            ParserError::Other { inner } => inner.fmt(f),
            ParserError::NoParsers(path) => {
                write!(
                    f,
                    "No parser that could handle {} was found!",
                    path.display()
                )
            }
        }
    }
}

impl Error for ParserError {}

impl PartialEq for ParserError {
    fn eq(&self, other: &ParserError) -> bool {
        match (self, other) {
            (ParserError::Deserialize(val), ParserError::Deserialize(other_val)) => {
                val.eq(other_val)
            }
            (ParserError::Loading(err), ParserError::Loading(other_err)) => {
                format!("{}", err).eq(&format!("{}", other_err))
            }
            (ParserError::Other { inner: err }, ParserError::Other { inner: other_err }) => {
                format!("{}", err).eq(&format!("{}", other_err))
            }
            (ParserError::NoParsers(path), ParserError::NoParsers(other_path)) => {
                path.eq(other_path)
            }
            _ => false,
        }
    }
}
