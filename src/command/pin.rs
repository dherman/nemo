use structopt::StructOpt;

use jetson_core::error::ErrorDetails;
use jetson_core::session::{ActivityKind, Session};
use jetson_core::tool::ToolSpec;
use jetson_core::version::VersionSpec;
use jetson_fail::{throw, ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Pin {
    /// The tool to install, e.g. `node` or `npm` or `yarn`
    tool: String,

    /// The version of the tool to install, e.g. `1.2.3` or `latest`
    version: Option<String>,
}

impl Command for Pin {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Pin);

        let version = match self.version {
            Some(version_string) => VersionSpec::parse(&version_string)?,
            None => VersionSpec::default(),
        };

        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        match tool {
            ToolSpec::Node(version) => session.pin_node(&version)?,
            ToolSpec::Yarn(version) => session.pin_yarn(&version)?,
            // ISSUE(#292): Implement install for npm
            ToolSpec::Npm(_version) => unimplemented!("Pinning npm is not supported yet"),
            ToolSpec::Package(name, _version) => {
                throw!(ErrorDetails::CannotPinPackage { package: name })
            }
        }

        session.add_event_end(ActivityKind::Pin, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
