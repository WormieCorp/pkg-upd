// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use pkg_data::prelude::PackageMetadata;

use crate::rules::{MessageType, RuleHandler, RuleKind, RuleMessage};

pub struct MaintainersNotEmptyRequirement;

impl RuleHandler<PackageMetadata> for MaintainersNotEmptyRequirement {
    #[inline(always)]
    fn should_validate(_: &RuleKind) -> bool {
        true
    }

    fn validate(metadata: &PackageMetadata) -> Result<(), RuleMessage> {
        let maintainers = metadata.maintainers();

        if maintainers.iter().all(|m| m.is_empty()) {
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "At least 1 maintainer must be specified for the package!".into(),
                package_manager: "",
            })
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest(kind, case(RuleKind::Community), case(RuleKind::Core))]
    fn should_validate_should_always_be_true(kind: RuleKind) {
        assert!(MaintainersNotEmptyRequirement::should_validate(&kind));
    }

    #[test]
    fn validate_should_return_rule_message_on_empty_maintainers() {
        let mut data = PackageMetadata::default();
        let maintainers: [&str; 0] = [];
        data.set_maintainers(&maintainers);

        let result = MaintainersNotEmptyRequirement::validate(&data);

        assert_eq!(
            result,
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "At least 1 maintainer must be specified for the package!".into(),
                package_manager: ""
            })
        )
    }

    #[test]
    fn validate_should_return_rule_message_when_all_items_is_empty() {
        let mut data = PackageMetadata::default();
        data.set_maintainers(&["", "", ""]);

        let result = MaintainersNotEmptyRequirement::validate(&data);

        assert_eq!(
            result,
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "At least 1 maintainer must be specified for the package!".into(),
                package_manager: ""
            })
        )
    }

    #[test]
    fn validate_should_not_return_message_on_non_empty_array() {
        let mut data = PackageMetadata::default();
        data.set_maintainers(&["AdmiringWorm", "Chocolatey"]);

        let result = MaintainersNotEmptyRequirement::validate(&data);

        assert_eq!(result, Ok(()))
    }
}
