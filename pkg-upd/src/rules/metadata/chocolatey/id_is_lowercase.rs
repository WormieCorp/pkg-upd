// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

use pkg_data::prelude::PackageMetadata;

use crate::rules::{MessageType, RuleHandler, RuleKind, RuleMessage};

pub struct IdIsLowercaseNote;

impl RuleHandler<PackageMetadata> for IdIsLowercaseNote {
    fn should_validate(rule_kind: &RuleKind) -> bool {
        rule_kind == &RuleKind::Community
    }

    fn validate(data: &PackageMetadata) -> std::result::Result<(), RuleMessage> {
        let id = data.id();

        if id.chars().any(|ch| ch.is_uppercase()) {
            Err(RuleMessage {
                message_type: MessageType::Note,
                message: "The identifier contains upper case characters. If this is a new \
                          package, it should only contain characters in lower case!"
                    .into(),
                package_manager: "choco",
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
    fn should_validate_should_be_true_for_community() {
        assert!(IdIsLowercaseNote::should_validate(&RuleKind::Community))
    }

    #[rstest(kind, case(RuleKind::Core))]
    fn should_validate_should_be_false(kind: RuleKind) {
        assert!(!IdIsLowercaseNote::should_validate(&kind))
    }

    #[test]
    fn validate_should_return_rule_message_on_uppercase_letter() {
        let data = PackageMetadata::new("test-PackAGE");

        let result = IdIsLowercaseNote::validate(&data);

        assert_eq!(
            result,
            Err(RuleMessage {
                message_type: MessageType::Note,
                message: "The identifier contains upper case characters. If this is a new \
                          package, it should only contain characters in lower case!"
                    .into(),
                package_manager: "choco",
            })
        )
    }

    #[test]
    fn validate_should_not_return_message_on_all_lowercase_letters() {
        let data = PackageMetadata::new("test-package");

        let result = IdIsLowercaseNote::validate(&data);

        assert_eq!(result, Ok(()))
    }
}
