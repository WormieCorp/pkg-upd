// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains the necessary logic of creating/generating the files that should be
//! used in a package.

#![deny(missing_docs)]

#[cfg(feature = "chocolatey")]
mod chocolatey;

use std::path::Path;

/// Defines a trait that allows creating necessary package files based on the
/// wanted generator information.
pub trait PackageGenerator {
    /// Generates the files that are expected from the specified metadata.
    fn generate(&self, work_dir: &Path) -> Result<(), Box<dyn std::error::Error>>;
}
