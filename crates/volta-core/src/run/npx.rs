use std::ffi::{OsStr, OsString};

use super::ToolCommand;
use crate::error::ErrorDetails;
use crate::platform::Source;
use crate::session::{ActivityKind, Session};
use crate::style::tool_version;
use crate::version::parse_version;

use log::debug;
use volta_fail::Fallible;

pub(crate) fn command<A>(args: A, session: &mut Session) -> Fallible<ToolCommand>
where
    A: IntoIterator<Item = OsString>,
{
    session.add_event_start(ActivityKind::Npx);

    match session.current_platform()? {
        Some(platform) => {
            let image = platform.checkout(session)?;

            // npx was only included with npm 5.2.0 and higher. If the npm version is less than that, we
            // should include a helpful error message
            let required_npm = parse_version("5.2.0")?;
            if *image.npm() >= required_npm {
                let source = match image.source() {
                    Source::Project | Source::ProjectNodeDefaultYarn => "project",
                    Source::Default => "default",
                };
                let version = tool_version("npx", image.npm());
                debug!("Using {} from {} configuration", version, source);

                let path = image.path()?;
                Ok(ToolCommand::direct(OsStr::new("npx"), args, &path))
            } else {
                Err(ErrorDetails::NpxNotAvailable {
                    version: image.npm().to_string(),
                }
                .into())
            }
        }
        None => {
            debug!("Could not find Volta-managed npx, delegating to system");
            ToolCommand::passthrough(OsStr::new("npx"), args, ErrorDetails::NoPlatform)
        }
    }
}
