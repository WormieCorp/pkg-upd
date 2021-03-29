// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

mod metadata;

use std::fmt::Display;
use std::str::FromStr;

use pkg_data::metadata::PackageMetadata;

#[macro_export(local_inner_macros)]
macro_rules! call_rules {
    ($msgs:expr,$rule_kind:expr,$data:expr,$($rule:path),*) => {
        use crate::rules::RuleHandler;

        $(
            if <$rule>::should_validate($rule_kind) {
                if let Err(msg) = <$rule>::validate($data) {
                    $msgs.push(msg)
                }
            }
        )*
    };
}

pub fn validate_metadata(
    data: &PackageMetadata,
    rule_kind: &RuleKind,
) -> Result<(), Vec<RuleMessage>> {
    let mut msgs = vec![];

    metadata::run_validation(&mut msgs, data, rule_kind);

    if msgs.is_empty() { Ok(()) } else { Err(msgs) }
}

pub trait RuleHandler<T> {
    fn should_validate(rule_type: &RuleKind) -> bool;
    fn validate(data: &T) -> Result<(), RuleMessage>;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MessageType {
    Requirement,
    Guideline,
    Suggestion,
    Note,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuleMessage {
    pub message_type: MessageType,
    pub package_manager: &'static str,
    pub message: String,
}

#[derive(Debug, PartialEq)]
pub enum RuleKind {
    Core,
    Community,
}

impl FromStr for RuleKind {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, <Self as std::str::FromStr>::Err> {
        let value_lower = value.to_lowercase();

        if value_lower == "core" {
            Ok(RuleKind::Core)
        } else if value_lower == "community" {
            Ok(RuleKind::Community)
        } else {
            Err(format!("{} is not a valid rule!", value))
        }
    }
}

impl Default for RuleKind {
    fn default() -> Self {
        RuleKind::Core
    }
}

impl Display for RuleKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            RuleKind::Core => f.write_str("core"),
            RuleKind::Community => f.write_str("community"),
        }
    }
}
