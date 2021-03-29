// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

mod id_is_lowercase;

use pkg_data::metadata::PackageMetadata;

use crate::call_rules;
use crate::rules::{RuleKind, RuleMessage};

pub fn run_validation(msgs: &mut Vec<RuleMessage>, data: &PackageMetadata, rule_kind: &RuleKind) {
    call_rules!(msgs, rule_kind, data, id_is_lowercase::IdIsLowercaseNote);
}
