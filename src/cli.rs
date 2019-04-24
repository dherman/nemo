use structopt::StructOpt;

use crate::command::{self, Command};
use jetson_core::path;
use jetson_core::session::Session;
use jetson_fail::{ExitCode, Fallible};

#[derive(StructOpt)]
#[structopt(
    name = "Jetson",
    about = "The hassle-free JavaScript toolchain manager",
    author = "",
    long_about = "The hassle-free JavaScript toolchain manager

    To install a tool in your toolchain, use `jetson install`.
    To pin your project's runtime or package manager, use `jetson pin`.",
    raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"),
    raw(global_setting = "structopt::clap::AppSettings::ColorAlways"),
    raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"),
    raw(global_setting = "structopt::clap::AppSettings::DisableVersion"),
    raw(global_setting = "structopt::clap::AppSettings::DontCollapseArgsInUsage"),
    raw(global_setting = "structopt::clap::AppSettings::VersionlessSubcommands")
)]
pub(crate) struct Jetson {
    #[structopt(subcommand)]
    pub(crate) command: Option<Subcommand>,

    #[structopt(long = "verbose", help = "Enables verbose diagnostics", global = true)]
    pub(crate) verbose: bool,

    #[structopt(
        short = "v",
        long = "version",
        help = "Prints the current version of Jetson"
    )]
    pub(crate) version: bool,
}

impl Jetson {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        path::ensure_jetson_dirs_exist()?;
        if self.version {
            println!("{}", env!("CARGO_PKG_VERSION"));
            Ok(ExitCode::Success)
        } else if let Some(command) = self.command {
            command.run(session)
        } else {
            Jetson::from_iter(["jetson", "help"].iter()).run(session)
        }
    }
}

#[derive(StructOpt)]
pub(crate) enum Subcommand {
    /// Fetches a tool to the local machine
    #[structopt(name = "fetch", author = "", version = "")]
    Fetch(command::Fetch),

    /// Installs a tool in your toolchain
    #[structopt(name = "install", author = "", version = "")]
    Install(command::Install),

    /// Uninstalls a tool from your toolchain
    #[structopt(name = "uninstall", author = "", version = "")]
    Uninstall(command::Uninstall),

    /// Pins your project's runtime or package manager
    #[structopt(name = "pin", author = "", version = "")]
    Pin(command::Pin),

    /// Displays the currently activated Node version
    #[structopt(name = "current", author = "", version = "")]
    Current(command::Current),

    /// Disables Jetson in the current shell
    #[structopt(
        name = "deactivate",
        author = "",
        version = "",
        raw(setting = "structopt::clap::AppSettings::Hidden")
    )]
    Deactivate(command::Deactivate),

    /// Re-enables Jetson in the current shell
    #[structopt(
        name = "activate",
        author = "",
        version = "",
        raw(setting = "structopt::clap::AppSettings::Hidden")
    )]
    Activate(command::Activate),

    /// Generates Jetson completions
    #[structopt(
        name = "completions",
        author = "",
        version = "",
        raw(setting = "structopt::clap::AppSettings::ArgRequiredElseHelp"),
        long_about = "Generates Jetson completions

By default, completions will be generated for the value of your current shell,
shell, i.e. the value of `SHELL`. If you set the `<shell>` option, completions
will be generated for that shell instead.

If you specify a directory, the completions will be written to a file there;
otherwise, they will be written to `stdout`.
    "
    )]
    Completions(command::Completions),

    /// Locates the actual binary that will be called by Jetson
    #[structopt(name = "which", author = "", version = "")]
    Which(command::Which),

    #[structopt(
        name = "use",
        author = "",
        version = "",
        template = "{usage}",
        raw(
            usage = "crate::command::r#use::USAGE",
            setting = "structopt::clap::AppSettings::Hidden"
        )
    )]
    Use(command::Use),
}

impl Subcommand {
    pub(crate) fn run(self, session: &mut Session) -> Fallible<ExitCode> {
        match self {
            Subcommand::Fetch(fetch) => fetch.run(session),
            Subcommand::Install(install) => install.run(session),
            Subcommand::Uninstall(uninstall) => uninstall.run(session),
            Subcommand::Pin(pin) => pin.run(session),
            Subcommand::Current(current) => current.run(session),
            Subcommand::Deactivate(deactivate) => deactivate.run(session),
            Subcommand::Activate(activate) => activate.run(session),
            Subcommand::Completions(completions) => completions.run(session),
            Subcommand::Which(which) => which.run(session),
            Subcommand::Use(r#use) => r#use.run(session),
        }
    }
}
