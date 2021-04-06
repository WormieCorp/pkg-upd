// Copyright (c) 2021 Kim J. Nordmo and WormieCorp.
// Licensed under the MIT license. See LICENSE.txt file in the project

#![cfg_attr(docsrs, doc(cfg(any(feature = "powershell"))))]

use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};

use crate::runners::{RunnerCombiner, RunnerData, ScriptRunner};

lazy_static! {
    static ref POWERSHELL_EXEC: PathBuf = {
        const NAMES: [&str; 3] = ["pwsh", "pwsh.exe", "powershell.exe"];
        let paths = get_env_paths();

        for name in NAMES.iter() {
            for path in paths.iter() {
                let path = PathBuf::from(path).join(name).canonicalize();

                if let Ok(path) = path {
                    return path;
                }
            }
        }

        PathBuf::new()
    };
}

fn get_powershell_path() -> &'static Path {
    POWERSHELL_EXEC.as_path()
}

pub struct PowershellRunner;

impl ScriptRunner for PowershellRunner {
    fn can_run(&self, script_path: &Path) -> bool {
        script_path.to_string_lossy().ends_with(".ps1")
    }

    fn run<'a, T: RunnerCombiner + Debug>(
        &self,
        cwd: &'a Path,
        script: PathBuf,
        data: &'a mut T,
    ) -> Result<(), String> {
        let path = get_powershell_path();

        if !path.is_file() {
            error!("No powershell executable was found!");
            return Err("No powershell executable was found!!".into());
        }
        let runner_data = serde_json::to_string(&data.to_runner_data()).unwrap();
        let script = script.canonicalize().unwrap();
        let override_script = if cfg!(windows) {
            "Set-ExecutionPolicy Bypass -Scope Process -Force;"
        } else {
            ""
        };
        let runner_template = format!(
            "$ErrorActiorPreference = 'Stop'; $InformationPreference = 'Continue'; \
             $VerbosePreference = 'Continue'; $DebugPreference = 'Continue'; {} $data = (\"{}\" | \
             ConvertFrom-Json -AsHashtable); [int]$exitCode = 0; try {{ {} $data; [int]$exitCode \
             = $LASTEXITCODE; }} catch {{ Write-Error $_; if ($LASTEXITCODE -eq 0) {{ \
             [int]$exitCode = 1; }} }}; Write-Host \"## AER-SCRIPT-RUNNER:START ##\"; Write-Host \
             ($data | ConvertTo-Json); Write-Host \"## AER-SCRIPT-RUNNER:END ##\"; if ($exitCode \
             -ne 0) {{ throw \"Non-Zero exit code: $exitCode\"; }}",
            override_script,
            runner_data.replace("\"", "`\""),
            script.display()
        );
        trace!("Data before running: {:?}", data);
        info!("Running script: {}", script.display());

        let cmd = Command::new(path)
            .current_dir(cwd)
            .env("POWERSHELL_TELEMETRY_OPTOUT", "1")
            .args(&[
                "-ExecutionPolicy",
                "Bypass",
                "-NoProfile",
                "-NonInteractive",
                "-Command",
            ])
            .arg(runner_template)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("failed to execute powershell script");

        let cmd = cmd.wait_with_output();

        if let Err(cmd) = cmd {
            error!("{}", cmd);
            return Err(format!("The running of powershell failed with '{}'", cmd));
        }

        let cmd = cmd.unwrap();
        if !cmd.status.success() {
            error!(
                "Powershell Script runner returned {} error code!",
                cmd.status
            )
        }

        let mut run_data = String::new();
        {
            let stdout = String::from_utf8_lossy(&cmd.stdout);
            let mut in_data = false;
            debug!("AER-SCRIPT-RUNNER STDOUT:");

            for line in stdout.lines() {
                match line.trim() {
                    "## AER-SCRIPT-RUNNER:START ##" => in_data = true,
                    "## AER-SCRIPT-RUNNER:END ##" => in_data = false,
                    line => {
                        if in_data {
                            run_data.push_str(line);
                        } else if line.starts_with("WARNING:") {
                            warn!("{}", line);
                        } else {
                            debug!("{}", line);
                        }
                    }
                }
            }
        }

        {
            let stderr = String::from_utf8_lossy(&cmd.stderr);
            debug!("AER-SCRIPT-RUNNER STDERR:");
            let mut fail = false;

            for line in stderr.lines() {
                fail = true;
                if line.trim().starts_with("WARNING:") {
                    warn!("{}", line);
                } else {
                    error!("{}", line);
                }
            }

            if fail {
                return Err(format!(
                    "An exception occurred when running the PowerShell script!\n{}",
                    stderr
                ));
            }
        }

        match serde_json::from_str::<RunnerData>(&run_data) {
            Ok(package_data) => {
                data.from_runner_data(package_data);
                trace!("Data after running: {:?}", data);
                Ok(())
            }
            Err(err) => {
                error!("{}", err);
                Err(format!(
                    "Deserializing script runner data failed with: {}",
                    err
                ))
            }
        }
    }
}

fn get_env_paths() -> Vec<String> {
    let split = if cfg!(windows) { ';' } else { ':' };

    if let Ok(path) = std::env::var("PATH") {
        path.split(split).map(String::from).collect()
    } else {
        vec![]
    }
}

#[cfg(test)]
mod tests {
    use aer_data::prelude::*;
    use rstest::rstest;

    use super::*;

    #[test]
    fn can_run_should_return_true_on_powershell_scripts() {
        let runner = PowershellRunner;
        let script = PathBuf::from("./test.ps1");

        let result = runner.can_run(&script);

        assert!(result);
    }

    #[rstest(
        name,
        case("my-test.cmd"),
        case("test-file.bat"),
        case("no.sh"),
        case("binary.exe")
    )]
    fn can_run_should_return_false_for_non_powershell_scripts(name: &str) {
        let runner = PowershellRunner;
        let script = PathBuf::from("./").join(name);

        let result = runner.can_run(&script);

        assert!(!result);
    }

    #[test]
    #[should_panic(expected = "An exception occurred when running the PowerShell script!")]
    fn run_should_return_error_when_file_is_directory() {
        let runner = PowershellRunner;
        let dir = PathBuf::from("src");
        let mut data = PackageData::new("test");

        let _ = runner.run(&PathBuf::from("."), dir, &mut data).unwrap();
    }

    #[rstest(name, case("empty-run.ps1"), case("empty-run-with-data.ps1"))]
    fn run_should_succeed_when_running_an_empty_powershell_script(name: &str) {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1").join(name);
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn run_should_not_use_changes_to_identifier() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-identifier.ps1");
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(data.metadata().id(), "test");
    }

    #[test]
    fn run_should_allow_changes_to_summary() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-summary.ps1");
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(
            data.metadata().summary,
            "The summary was changed to something else"
        );
    }

    #[test]
    fn run_should_allow_changes_to_project_url() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-project_url.ps1");
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(
            data.metadata().project_url(),
            &Url::parse("https://github.com/WormieCorp/aer").unwrap()
        );
    }

    #[test]
    fn run_should_allow_changes_to_license_expression() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-license-expression.ps1");
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(
            data.metadata().license(),
            &LicenseType::Expression("Apache-2.0".into())
        );
    }

    #[test]
    fn run_should_allow_changes_to_license_url() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-license-url.ps1");
        let mut data = PackageData::new("test");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(
            data.metadata().license(),
            &LicenseType::Location(
                Url::parse(
                    "https://github.com/AdmiringWorm/chocolatey-packages/blob/master/LICENSE.txt"
                )
                .unwrap()
            )
        );
    }

    #[test]
    fn run_should_allow_changing_license_expression_and_url() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/change-license-full.ps1");
        let mut data = PackageData::new("codecov");

        let result = runner.run(&PathBuf::from("."), path, &mut data);

        assert_eq!(result, Ok(()));
        assert_eq!(
            data.metadata().license(),
            &LicenseType::ExpressionAndLocation {
                url: Url::parse(
                    "https://github.com/AdmiringWorm/chocolatey-packages/blob/master/LICENSE.txt"
                )
                .unwrap(),
                expression: "Apache-2.0".into()
            }
        );
    }

    #[test]
    #[should_panic(expected = "An exception occurred when running the PowerShell script!")]
    fn run_should_return_error_when_an_exception_occurrs() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/with-exception.ps1");
        let mut data = PackageData::new("ansible");

        let _ = runner.run(&PathBuf::from("."), path, &mut data).unwrap();
    }

    #[test]
    #[should_panic(expected = "An exception occurred when running the PowerShell script!")]
    fn run_should_return_error_when_script_exits_with_non_zero_exit_code() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/exit-code.ps1");
        let mut data = PackageData::new("ansible");

        let _ = runner.run(&PathBuf::from("."), path, &mut data).unwrap();
    }

    #[test]
    #[should_panic(expected = "An exception occurred when running the PowerShell script!")]
    fn run_should_return_error_when_script_is_invalid() {
        let runner = PowershellRunner;
        let path = PathBuf::from("test-data/ps1/invalid-powershell.ps1");
        let mut data = PackageData::new("ansible");

        let _ = runner.run(&PathBuf::from("."), path, &mut data).unwrap();
    }
}
