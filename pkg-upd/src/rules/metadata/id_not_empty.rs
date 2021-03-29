// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use pkg_data::metadata::PackageMetadata;

use crate::rules::{MessageType, RuleHandler, RuleKind, RuleMessage};

pub struct IdNotEmptyRequirement;

impl RuleHandler<PackageMetadata> for IdNotEmptyRequirement {
    #[inline(always)]
    fn should_validate(_: &RuleKind) -> bool {
        true
    }

    fn validate(data: &PackageMetadata) -> Result<(), RuleMessage> {
        if data.id().trim().is_empty() {
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "A identifier can not be empty!".into(),
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

    #[test]
    fn should_validate_should_always_be_true() {
        assert!(IdNotEmptyRequirement::should_validate(&RuleKind::Core));
        assert!(IdNotEmptyRequirement::should_validate(&RuleKind::Community));
    }

    #[rstest(id, case(""), case("   "), case(" \n"), case("\r "), case("\r\n"))]
    fn validate_should_return_rule_message_on_empty_id(id: &'static str) {
        let data = PackageMetadata::new(id);

        let result = IdNotEmptyRequirement::validate(&data);

        assert_eq!(
            result,
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "A identifier can not be empty!".into(),
                package_manager: ""
            })
        );
    }

    #[test]
    fn validate_should_not_return_message_on_non_empty_id() {
        let data = PackageMetadata::new("test-id");

        let result = IdNotEmptyRequirement::validate(&data);

        assert_eq!(result, Ok(()));
    }
}
