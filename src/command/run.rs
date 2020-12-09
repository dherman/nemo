use std::collections::HashMap;
use std::ffi::OsString;

use crate::command::Command;
use crate::common::{Error, IntoResult};
use log::warn;
use structopt::StructOpt;
use volta_core::error::{report_error, ExitCode, Fallible};
use volta_core::platform::{CliPlatform, InheritOption};
use volta_core::run::execute_tool;
use volta_core::session::{ActivityKind, Session};
use volta_core::tool::{node, npm, yarn};

#[derive(Debug, StructOpt)]
pub(crate) struct Run {
    /// Set the custom Node version
    #[structopt(long = "node", value_name = "version")]
    node: Option<String>,

    /// Set the custom npm version
    #[structopt(long = "npm", value_name = "version", conflicts_with = "bundled_npm")]
    npm: Option<String>,

    /// Forces npm to be the version bundled with Node
    #[structopt(long = "bundled-npm", conflicts_with = "npm")]
    bundled_npm: bool,

    /// Set the custom Yarn version
    #[structopt(long = "yarn", value_name = "version", conflicts_with = "no_yarn")]
    yarn: Option<String>,

    /// Disables Yarn
    #[structopt(long = "no-yarn", conflicts_with = "yarn")]
    no_yarn: bool,

    /// Set an environment variable (can be used multiple times)
    #[structopt(long = "env", value_name = "NAME=value", raw(number_of_values = "1"))]
    envs: Vec<String>,

    #[structopt(parse(from_os_str))]
    /// The command to run
    command: OsString,

    #[structopt(parse(from_os_str))]
    /// Arguments to pass to the command
    args: Vec<OsString>,
}

impl Command for Run {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Run);

        let envs = self.parse_envs();
        let platform = self.parse_platform(session)?;

        match execute_tool(&self.command, &self.args, &envs, platform, session).into_result() {
            Ok(()) => {
                session.add_event_end(ActivityKind::Run, ExitCode::Success);
                Ok(ExitCode::Success)
            }
            Err(Error::Tool(code)) => {
                session.add_event_tool_end(ActivityKind::Run, code);
                Ok(ExitCode::ExecutionFailure)
            }
            Err(Error::Volta(err)) => {
                report_error(env!("CARGO_PKG_VERSION"), &err);
                session.add_event_error(ActivityKind::Run, &err);
                session.add_event_end(ActivityKind::Run, err.exit_code());
                Ok(err.exit_code())
            }
        }
    }
}

impl Run {
    /// Builds a CliPlatform from the provided cli options
    ///
    /// Will resolve a semver / tag version if necessary
    fn parse_platform(&self, session: &mut Session) -> Fallible<CliPlatform> {
        let node = self
            .node
            .as_ref()
            .map(|version| node::resolve(version.parse()?, session))
            .transpose()?;

        let npm = match (self.bundled_npm, &self.npm) {
            (true, _) => InheritOption::None,
            (false, None) => InheritOption::Inherit,
            (false, Some(version)) => match npm::resolve(version.parse()?, session)? {
                None => InheritOption::Inherit,
                Some(npm) => InheritOption::Some(npm),
            },
        };

        let yarn = match (self.no_yarn, &self.yarn) {
            (true, _) => InheritOption::None,
            (false, None) => InheritOption::Inherit,
            (false, Some(version)) => {
                InheritOption::Some(yarn::resolve(version.parse()?, session)?)
            }
        };

        Ok(CliPlatform { node, npm, yarn })
    }

    /// Convert the environment variable settings passed to the command line into a map
    ///
    /// We ignore any setting that doesn't have a value associated with it
    /// We also ignore the PATH environment variable as that is set when running a command
    fn parse_envs(&self) -> HashMap<&str, &str> {
        self.envs.iter().filter_map(|entry| {
            let mut key_value = entry.splitn(2, '=');

            match (key_value.next(), key_value.next()) {
                (None, _) => None,
                (Some(_), None) => None,
                (Some(key), _) if key.eq_ignore_ascii_case("PATH") => {
                    warn!("Ignoring {} environment variable as it will be overwritten when executing the command", key);
                    None
                }
                (Some(key), Some(value)) => Some((key, value)),
            }
        }).collect()
    }
}
