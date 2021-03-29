// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project
use pkg_data::prelude::PackageMetadata;

use crate::rules::{MessageType, RuleHandler, RuleKind, RuleMessage};

pub struct ProjectUrlNotLocalPathRequirement;

impl RuleHandler<PackageMetadata> for ProjectUrlNotLocalPathRequirement {
    #[inline(always)]
    fn should_validate(_: &RuleKind) -> bool {
        true
    }

    fn validate(data: &PackageMetadata) -> std::result::Result<(), RuleMessage> {
        let project_url = data.project_url();
        if !project_url.has_host()
            || project_url.to_file_path().is_ok()
            || project_url.scheme().to_lowercase() == "file"
        {
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "The project url can not be a local path!".into(),
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
        assert!(ProjectUrlNotLocalPathRequirement::should_validate(
            &RuleKind::Core
        ));
        assert!(ProjectUrlNotLocalPathRequirement::should_validate(
            &RuleKind::Community
        ));
    }

    #[rstest(
        url,
        case("file:///home/test/test-path"),
        case("file://C:/test-path"),
        case("file://localhost/etc/fstab"),
        case("file:///c:/WINDOWS/clock.avi"),
        case("file://localhost/c$/WINDOWS/clock.avi"),
        case("file://./sharename/path/to/the%20file.txt")
    )]
    fn validate_should_return_rule_message_on_local_paths(url: &'static str) {
        let mut data = PackageMetadata::new("valid-id");
        data.set_project_url(url);

        let result = ProjectUrlNotLocalPathRequirement::validate(&data);

        assert_eq!(
            result,
            Err(RuleMessage {
                message_type: MessageType::Requirement,
                message: "The project url can not be a local path!".into(),
                package_manager: ""
            })
        );
    }

    #[test]
    fn validate_should_not_return_any_messages_on_valid_url() {
        let mut data = PackageMetadata::new("valid-id");
        data.set_project_url("https://github.com");
        let result = ProjectUrlNotLocalPathRequirement::validate(&data);

        assert_eq!(result, Ok(()))
    }
}
