use structopt::StructOpt;

use jetson_core::session::{ActivityKind, Session};
use jetson_core::tool::ToolSpec;
use jetson_core::version::VersionSpec;
use jetson_fail::{ExitCode, Fallible};

use crate::command::Command;

#[derive(StructOpt)]
pub(crate) struct Install {
    /// The tool to install, e.g. `node` or `npm` or `yarn`
    tool: String,

    /// The version of the tool to install, e.g. `1.2.3` or `latest`
    version: Option<String>,
}

impl Command for Install {
    fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        session.add_event_start(ActivityKind::Install);

        let version = match self.version {
            Some(version_string) => VersionSpec::parse(version_string)?,
            None => VersionSpec::default(),
        };
        let tool = ToolSpec::from_str_and_version(&self.tool, version);

        tool.install(session)?;

        session.add_event_end(ActivityKind::Install, ExitCode::Success);
        Ok(ExitCode::Success)
    }
}
