// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

pub mod chocolatey;

pub trait FixVersion {
    fn add_fix(&mut self) -> Result<(), std::num::ParseIntError>;
}
