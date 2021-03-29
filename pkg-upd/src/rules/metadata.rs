// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

mod chocolatey;
mod id_not_empty;
mod maintainers_not_empty;
mod project_url_not_local_path;

use pkg_data::metadata::PackageMetadata;

use crate::call_rules;
use crate::rules::{RuleKind, RuleMessage};

pub fn run_validation(msgs: &mut Vec<RuleMessage>, data: &PackageMetadata, rule_kind: &RuleKind) {
    call_rules!(
        msgs,
        rule_kind,
        data,
        id_not_empty::IdNotEmptyRequirement,
        maintainers_not_empty::MaintainersNotEmptyRequirement,
        project_url_not_local_path::ProjectUrlNotLocalPathRequirement
    );

    chocolatey::run_validation(msgs, data, rule_kind);
}
