// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

//! Contains logic necessary for creating/generating package files for the
//! Chocolatey package manager.

#![cfg_attr(docsrs, doc(cfg(any(feature = "chocolatey"))))]

pub mod nuspec;
