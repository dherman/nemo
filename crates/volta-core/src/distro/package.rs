//! Provides types for installing packages to the user toolchain.

use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::{self, rename, write, File};
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str;

use atty::Stream;
use cfg_if::cfg_if;
use hex;
use log::{debug, info};
use semver::Version;
use sha1::{Digest, Sha1};
use tempfile::tempdir_in;

use crate::command::create_command;
use crate::distro::{download_tool_error, Distro, Fetched};
use crate::error::ErrorDetails;
use crate::fs::{
    delete_dir_error, dir_entry_match, ensure_containing_dir_exists, ensure_dir_does_not_exist,
    read_dir_eager, read_file_opt,
};
use crate::hook::ToolHooks;
use crate::inventory::Collection;
use crate::layout::layout;
use crate::manifest::Manifest;
use crate::platform::{Image, PlatformSpec};
use crate::session::Session;
use crate::shim;
use crate::style::{progress_bar, progress_spinner, tool_version};
use crate::tool::ToolSpec;
use crate::version::VersionSpec;
use archive::{Archive, Tarball};

cfg_if! {
    if #[cfg(windows)] {
        use cmdline_words_parser::StrExt;
        use regex::Regex;
        use std::io::{BufRead, BufReader};
    } else if #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
    }
}

use volta_fail::{throw, Fallible, ResultExt};

/// A provisioned Package distribution.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PackageDistro {
    pub name: String,
    pub shasum: String,
    pub tarball_url: String,
    pub version: Version,
    pub image_dir: PathBuf,
    pub shasum_file: PathBuf,
    pub distro_file: PathBuf,
}

/// A package version.
#[derive(Eq, PartialEq, Clone, Debug)]
pub struct PackageVersion {
    pub name: String,
    pub version: Version,
    // map of binary names to locations
    pub bins: HashMap<String, String>,
    image_dir: PathBuf,
}

/// Programs used to install packages.
enum Installer {
    Npm,
    Yarn,
}

/// Configuration information about an installed package.
///
/// This information will be stored in ~/.volta/tools/user/packages/<package>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "version": "1.4.0",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null
///   },
///   "bins": [
///     "cowsay",
///     "cowthink"
///   ]
/// }
pub struct PackageConfig {
    /// The package name
    pub name: String,
    /// The package version
    pub version: Version,
    /// The platform used to install this package
    pub platform: PlatformSpec,
    /// The binaries installed by this package
    pub bins: Vec<String>,
}

/// Configuration information about an installed binary from a package.
///
/// This information will be stored in ~/.volta/tools/user/bins/<bin-name>.json.
///
/// For an example, this looks like:
///
/// {
///   "name": "cowsay",
///   "package": "cowsay",
///   "version": "1.4.0",
///   "path": "./cli.js",
///   "platform": {
///     "node": {
///       "runtime": "11.10.1",
///       "npm": "6.7.0"
///     },
///     "yarn": null,
///     "loader": {
///       "exe": "node",
///       "args": []
///     }
///   }
/// }
pub struct BinConfig {
    /// The binary name
    pub name: String,
    /// The package that installed this binary
    pub package: String,
    /// The package version
    pub version: Version,
    /// The relative path of the binary in the installed package
    pub path: String,
    /// The platform used to install this binary
    pub platform: PlatformSpec,
    /// The loader information for the script, if any
    pub loader: Option<BinLoader>,
}

/// Information about the Shebang script loader (e.g. `#!/usr/bin/env node`)
///
/// Only important for Windows at the moment, as Windows does not natively understand script
/// loaders, so we need to provide that behavior when calling a script that uses one
pub struct BinLoader {
    /// The command used to run a script
    pub command: String,
    /// Any additional arguments specified for the loader
    pub args: Vec<String>,
}

impl Distro for PackageDistro {
    type VersionDetails = PackageVersion;
    type ResolvedVersion = PackageEntry;

    fn new(
        name: &str,
        entry: Self::ResolvedVersion,
        _hooks: Option<&ToolHooks<Self>>,
    ) -> Fallible<Self> {
        let version = entry.version;
        let layout = layout()?;
        Ok(PackageDistro {
            name: name.to_string(),
            shasum: entry.shasum,
            version: version.clone(),
            tarball_url: entry.tarball,
            image_dir: layout.user.package_image_dir(&name, &version.to_string()),
            distro_file: layout.user.package_distro_file(&name, &version.to_string()),
            shasum_file: layout
                .user
                .package_distro_shasum(&name, &version.to_string()),
        })
    }

    // Fetches and unpacks the PackageDistro
    fn fetch(self, _collection: &Collection<Self>) -> Fallible<Fetched<PackageVersion>> {
        // don't need to fetch if the package is already installed
        if self.is_installed() {
            return Ok(Fetched::Installed(PackageVersion::new(
                self.name.clone(),
                self.version.clone(),
                self.generate_bin_map()?,
            )?));
        }

        let archive = self.load_or_fetch_archive()?;

        let tmp_root = path::tmp_dir()?;
        let temp = tempdir_in(&tmp_root)
            .with_context(|_| ErrorDetails::CreateTempDirError { in_dir: tmp_root })?;
        self.log_unpacking(&temp.path().display());

        let bar = progress_bar(
            archive.origin(),
            &tool_version(&self.name, &self.version),
            archive
                .uncompressed_size()
                .unwrap_or(archive.compressed_size()),
        );

<<<<<<< HEAD
=======
        let layout = layout()?;
        let tmp_root = layout.user.tmp_dir();
        let temp = tempdir_in(&tmp_root).with_context(|_| ErrorDetails::CreateTempDirError {
            in_dir: tmp_root.to_string_lossy().to_string(),
        })?;
>>>>>>> Replace `notion_core::path` with the layout module!
        archive
            .unpack(temp.path(), &mut |_, read| {
                bar.inc(read as u64);
            })
            .with_context(|_| ErrorDetails::UnpackArchiveError {
                tool: self.name.clone(),
                version: self.version.to_string(),
            })?;

        // ensure that the dir where this will be unpacked exists
        ensure_containing_dir_exists(&self.image_dir)?;
        // and ensure that the target directory does not exist
        ensure_dir_does_not_exist(&self.image_dir)?;

        let unpack_dir = find_unpack_dir(temp.path())?;
        rename(&unpack_dir, &self.image_dir).with_context(|_| {
            ErrorDetails::SetupToolImageError {
                tool: self.name.clone(),
                version: self.version.to_string(),
                dir: self.image_dir.clone(),
            }
        })?;

        // save the shasum in a file
        write(&self.shasum_file, self.shasum.as_bytes()).with_context(|_| {
            ErrorDetails::WritePackageShasumError {
                package: self.name.clone(),
                version: self.version.to_string(),
                file: self.shasum_file.to_owned(),
            }
        })?;

        bar.finish_and_clear();

        // Note: We write this after the progress bar is finished to avoid display bugs with re-renders of the progress
        self.log_installing();
        Ok(Fetched::Now(PackageVersion::new(
            self.name.clone(),
            self.version.clone(),
            self.generate_bin_map()?,
        )?))
    }

    fn version(&self) -> &Version {
        &self.version
    }
}

impl PackageDistro {
    /// Loads the package tarball from disk, or fetches from URL.
    fn load_or_fetch_archive(&self) -> Fallible<Box<Archive>> {
        // try to use existing downloaded package
        if let Some(archive) = self.load_cached_archive() {
            debug!(
                "Loading {} from cached archive at {}",
                tool_version(&self.name, &self.version),
                self.distro_file.display()
            );
            Ok(archive)
        } else {
            // otherwise have to download
            ensure_containing_dir_exists(&self.distro_file)?;
            debug!(
                "Downloading {} from {}",
                tool_version(&self.name, &self.version),
                &self.tarball_url
            );

            Tarball::fetch(&self.tarball_url, &self.distro_file).with_context(download_tool_error(
                ToolSpec::Package(self.name.to_string(), VersionSpec::exact(&self.version)),
                self.tarball_url.to_string(),
            ))
        }
    }

    /// Verify downloaded package, returning an Archive if it is ok.
    fn load_cached_archive(&self) -> Option<Box<dyn Archive>> {
        let mut distro = File::open(&self.distro_file).ok()?;
        let stored_shasum = read_file_opt(&self.shasum_file).ok()??; // `??`: Err *or* None -> None

        let mut buffer = Vec::new();
        distro.read_to_end(&mut buffer).ok()?;

        // calculate the shasum
        let mut hasher = Sha1::new();
        hasher.input(buffer);
        let result = hasher.result();
        let calculated_shasum = hex::encode(&result);

        if stored_shasum != calculated_shasum {
            return None;
        }

        distro.seek(SeekFrom::Start(0)).ok()?;
        Tarball::load(distro).ok()
    }

    fn is_installed(&self) -> bool {
        // check that package config file contains the same version
        // (that is written after a package has been installed)
        if let Ok(layout) = layout() {
            let pkg_config_file = layout.user.user_package_config_file(&self.name);
            if let Ok(package_config) = PackageConfig::from_file(&pkg_config_file) {
                return package_config.version == self.version;
            }
        }
        false
    }

    fn generate_bin_map(&self) -> Fallible<HashMap<String, String>> {
        let pkg_info = Manifest::for_dir(&self.image_dir)?;
        let bin_map = pkg_info.bin;
        if bin_map.is_empty() {
            throw!(ErrorDetails::NoPackageExecutables);
        }

        for (bin_name, _bin_path) in bin_map.iter() {
            // check for conflicts with installed bins
            // some packages may install bins with the same name
            let bin_config_file = layout()?.user.user_tool_bin_config(&bin_name);
            if bin_config_file.exists() {
                let bin_config = BinConfig::from_file(bin_config_file)?;
                // if the bin was installed by the package that is currently being installed,
                // that's ok - otherwise it's an error
                if self.name != bin_config.package {
                    throw!(ErrorDetails::BinaryAlreadyInstalled {
                        bin_name: bin_name.to_string(),
                        existing_package: bin_config.package,
                        new_package: self.name.clone(),
                    });
                }
            }
        }

        Ok(bin_map)
    }

    fn log_unpacking<D>(&self, path: &D)
    where
        D: std::fmt::Display,
    {
        debug!(
            "Unpacking {} in {}",
            tool_version(&self.name, &self.version),
            path
        );
    }

    fn log_installing(&self) {
        debug!(
            "Installing {} in {}",
            tool_version(&self.name, &self.version),
            self.image_dir.display()
        );
    }
}

// Figure out the unpacked package directory name dynamically, because
// packages typically extract to a "package" directory, but not always
fn find_unpack_dir(in_dir: &Path) -> Fallible<PathBuf> {
    let dirs: Vec<_> = read_dir_eager(in_dir)
        .with_context(|_| ErrorDetails::PackageUnpackError)?
        .collect();

    // if there is only one directory, return that
    if let [(entry, metadata)] = dirs.as_slice() {
        if metadata.is_dir() {
            return Ok(entry.path().to_path_buf());
        }
    }
    // there is more than just a single directory here, something is wrong
    Err(ErrorDetails::PackageUnpackError.into())
}

impl PackageVersion {
    pub fn new(name: String, version: Version, bins: HashMap<String, String>) -> Fallible<Self> {
        let layout = layout()?;
        let image_dir = layout.user.package_image_dir(&name, &version.to_string());
        Ok(PackageVersion {
            name,
            version,
            bins,
            image_dir,
        })
    }

    // parse the "engines" string to a VersionSpec, for matching against available Node versions
    pub fn engines_spec(&self) -> Fallible<VersionSpec> {
        let manifest = Manifest::for_dir(&self.image_dir)?;
        // if nothing specified, can use any version of Node
        let engines = match manifest.engines() {
            Some(engines) => {
                debug!(
                    "Found 'engines.node' specification for {}: {}",
                    tool_version(&self.name, &self.version),
                    &engines
                );
                engines
            }
            None => {
                debug!(
                    "No 'engines.node' found for {}, using latest",
                    tool_version(&self.name, &self.version)
                );
                String::from("*")
            }
        };
        let spec = VersionSpec::parse_requirements(engines)?;
        Ok(VersionSpec::Semver(spec))
    }

    pub fn install(&self, platform: &PlatformSpec, session: &mut Session) -> Fallible<()> {
        let image = platform.checkout(session)?;
        // use yarn if it is installed, otherwise default to npm
        let installer = if image.yarn.is_some() {
            Installer::Yarn
        } else {
            Installer::Npm
        };

        let mut command =
            install_command_for(installer, self.image_dir.as_os_str(), &image.path()?);
        self.log_installing_dependencies(&command);

        let spinner = progress_spinner(&format!(
            "Installing dependencies for {}",
            tool_version(&self.name, &self.version)
        ));
        let output = command
            .output()
            .with_context(|_| ErrorDetails::PackageInstallFailed)?;
        spinner.finish_and_clear();

        self.log_dependency_install_stderr(&output.stderr);
        self.log_dependency_install_stdout(&output.stdout);

        if !output.status.success() {
            throw!(ErrorDetails::PackageInstallFailed);
        }

        self.write_config_and_shims(&platform)?;

        Ok(())
    }

    fn package_config(&self, platform_spec: &PlatformSpec) -> PackageConfig {
        PackageConfig {
            name: self.name.to_string(),
            version: self.version.clone(),
            platform: platform_spec.clone(),
            bins: self
                .bins
                .iter()
                .map(|(name, _path)| name.to_string())
                .collect(),
        }
    }

    fn bin_config(
        &self,
        bin_name: String,
        bin_path: String,
        platform_spec: PlatformSpec,
        loader: Option<BinLoader>,
    ) -> BinConfig {
        BinConfig {
            name: bin_name,
            package: self.name.to_string(),
            version: self.version.clone(),
            path: bin_path,
            platform: platform_spec,
            loader,
        }
    }

    fn write_config_and_shims(&self, platform_spec: &PlatformSpec) -> Fallible<()> {
        self.package_config(&platform_spec).to_serial().write()?;
        for (bin_name, bin_path) in self.bins.iter() {
            let full_path = bin_full_path(&self.name, &self.version, bin_name, bin_path)?;
            let loader = determine_script_loader(bin_name, &full_path)?;
            self.bin_config(
                bin_name.to_string(),
                bin_path.to_string(),
                platform_spec.clone(),
                loader,
            )
            .to_serial()
            .write()?;
            // create a link to the shim executable
            shim::create(&bin_name)?;

            // On Unix, ensure the executable file has correct permissions
            #[cfg(unix)]
            set_executable_permissions(&full_path).with_context(|_| {
                ErrorDetails::ExecutablePermissionsError {
                    bin: bin_name.clone(),
                }
            })?;
        }
        Ok(())
    }

    /// Uninstall the specified package.
    ///
    /// This removes:
    /// * the json config files
    /// * the shims
    /// * the unpacked and initialized package
    pub fn uninstall(name: &str) -> Fallible<()> {
        let layout = layout()?;

        // if the package config file exists, use that to remove any installed bins and shims
        let package_config_file = layout.user.user_package_config_file(&name);
        if package_config_file.exists() {
            let package_config = PackageConfig::from_file(&package_config_file)?;

            for bin_name in package_config.bins {
                PackageVersion::remove_config_and_shim(&bin_name, name)?;
            }

            fs::remove_file(&package_config_file)
                .with_context(delete_file_error(&package_config_file))?;
        } else {
            // there is no package config - check for orphaned binaries
            let user_bin_dir = layout.user.user_tool_bin_dir();
            if user_bin_dir.exists() {
                let orphaned_bins = binaries_from_package(name)?;
                for bin_name in orphaned_bins {
                    PackageVersion::remove_config_and_shim(&bin_name, name)?;
                }
            }
        }

        // if any unpacked and initialized packages exists, remove them
        let package_image_dir = layout.user.package_image_root_dir().join(&name);
        if package_image_dir.exists() {
            fs::remove_dir_all(&package_image_dir)
                .with_context(delete_dir_error(&package_image_dir))?;
        }

        Ok(())
    }

    fn remove_config_and_shim(bin_name: &str, name: &str) -> Fallible<()> {
        shim::delete(bin_name)?;
        let config_file = layout()?.user.user_tool_bin_config(&bin_name);
        fs::remove_file(&config_file).with_context(delete_file_error(&config_file))?;
        info!("Removed executable '{}' installed by '{}'", bin_name, name);
        Ok(())
    }

    fn log_installing_dependencies(&self, command: &Command) {
        debug!("Installing dependencies with command: {:?}", command);
    }

    fn log_dependency_install_stderr(&self, output: &Vec<u8>) {
        debug!("[install stderr]\n{}", String::from_utf8_lossy(output));
    }

    fn log_dependency_install_stdout(&self, output: &Vec<u8>) {
        debug!("[install stdout]\n{}", String::from_utf8_lossy(output));
    }
}

fn delete_file_error(file: &PathBuf) -> impl FnOnce(&io::Error) -> ErrorDetails {
    let file = file.to_path_buf();
    |_| ErrorDetails::DeleteFileError { file }
}

/// Reads the contents of a directory and returns a Vec containing the names of
/// all the binaries installed by the input package.
pub fn binaries_from_package(package: &str) -> Fallible<Vec<String>> {
    let layout = layout()?;
    let bin_config_dir = layout.user.user_tool_bin_dir();
    dir_entry_match(&bin_config_dir, |entry| {
        let path = entry.path();
        if let Ok(config) = BinConfig::from_file(path) {
            if config.package == package.to_string() {
                return Some(config.name);
            }
        };
        None
    })
    .with_context(|_| ErrorDetails::ReadBinConfigDirError {
        dir: bin_config_dir,
    })
}

impl Installer {
    pub fn cmd(&self) -> Command {
        match self {
            Installer::Npm => {
                let mut command = create_command("npm");
                command.args(&[
                    "install",
                    "--only=production",
                    "--loglevel=warn",
                    "--no-update-notifier",
                    "--no-audit",
                ]);

                if atty::is(Stream::Stdout) {
                    // npm won't detect the existence of a TTY since we are piping the output
                    // force the output to be colorized for when we send it to the user
                    command.arg("--color=always");
                }

                command
            }
            Installer::Yarn => {
                let mut command = create_command("yarn");
                command.args(&["install", "--production", "--non-interactive"]);
                command
            }
        }
    }
}

/// Information about a user tool.
/// This is defined in RFC#27: https://github.com/volta-cli/rfcs/pull/27
pub struct UserTool {
    pub bin_path: PathBuf,
    pub image: Image,
    pub loader: Option<BinLoader>,
}

impl UserTool {
    pub fn from_config(bin_config: BinConfig, session: &mut Session) -> Fallible<Self> {
        let bin_path = bin_full_path(
            &bin_config.package,
            &bin_config.version,
            &bin_config.name,
            &bin_config.path,
        )?;

        // If the user does not have yarn set in the platform for this binary, use the default
        // This is necessary because some tools (e.g. ember-cli with the --yarn option) invoke `yarn`
        let platform = match bin_config.platform.yarn {
            Some(_) => bin_config.platform,
            None => {
                let yarn = session
                    .user_platform()?
                    .and_then(|ref plat| plat.yarn.clone());
                PlatformSpec {
                    yarn,
                    ..bin_config.platform
                }
            }
        };

        Ok(UserTool {
            bin_path,
            image: platform.checkout(session)?,
            loader: bin_config.loader,
        })
    }

    pub fn from_name(tool_name: &str, session: &mut Session) -> Fallible<Option<UserTool>> {
        let bin_config_file = layout()?.user.user_tool_bin_config(tool_name);
        if bin_config_file.exists() {
            let bin_config = BinConfig::from_file(bin_config_file)?;
            UserTool::from_config(bin_config, session).map(Some)
        } else {
            Ok(None) // no config means the tool is not installed
        }
    }
}

fn bin_full_path<P>(
    package: &str,
    version: &Version,
    bin_name: &str,
    bin_path: P,
) -> Fallible<PathBuf>
where
    P: AsRef<Path>,
{
    // canonicalize because path is relative, and sometimes uses '.' char
    layout()?.user.package_image_dir(package, &version.to_string())
        .join(bin_path)
        .canonicalize()
        .with_context(|_| ErrorDetails::ExecutablePathError {
            command: bin_name.to_string(),
        })
}

/// Ensure that a given binary has 'executable' permissions on Unix, otherwise we won't be able to call it
/// On Windows, this isn't a concern as there is no concept of 'executable' permissions
#[cfg(unix)]
fn set_executable_permissions(bin: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(bin)?.permissions();
    let mode = permissions.mode();

    if mode & 0o111 != 0o111 {
        permissions.set_mode(mode | 0o111);
        fs::set_permissions(bin, permissions)
    } else {
        Ok(())
    }
}

/// On Unix, shebang loaders work correctly, so we don't need to bother storing loader information
#[cfg(unix)]
fn determine_script_loader(_bin_name: &str, _full_path: &Path) -> Fallible<Option<BinLoader>> {
    Ok(None)
}

/// On Windows, we need to read the executable and try to find a shebang loader
/// If it exists, we store the loader in the BinConfig so that the shim can execute it correctly
#[cfg(windows)]
fn determine_script_loader(bin_name: &str, full_path: &Path) -> Fallible<Option<BinLoader>> {
    let script =
        File::open(full_path).with_context(|_| ErrorDetails::DetermineBinaryLoaderError {
            bin: bin_name.to_string(),
        })?;
    if let Some(Ok(first_line)) = BufReader::new(script).lines().next() {
        // Note: Regex adapted from @zkochan/cmd-shim package used by Yarn
        // https://github.com/pnpm/cmd-shim/blob/bac160cc554e5157e4c5f5e595af30740be3519a/index.js#L42
        let re = Regex::new(r#"^#!\s*(?:/usr/bin/env)?\s*(?P<exe>[^ \t]+) ?(?P<args>.*)$"#)
            .expect("Regex is valid");
        if let Some(caps) = re.captures(&first_line) {
            let args = caps["args"]
                .to_string()
                .parse_cmdline_words()
                .map(|word| word.to_string())
                .collect();
            return Ok(Some(BinLoader {
                command: caps["exe"].to_string(),
                args,
            }));
        }
    }
    Ok(None)
}

/// Build a package install command using the specified directory and path
fn install_command_for(installer: Installer, in_dir: &OsStr, path_var: &OsStr) -> Command {
    let mut command = installer.cmd();
    command.current_dir(in_dir).env("PATH", path_var);
    command
}

/// Index of versions of a specific package.
pub struct PackageIndex {
    pub latest: Version,
    pub entries: Vec<PackageEntry>,
}

#[derive(Debug)]
pub struct PackageEntry {
    pub version: Version,
    pub tarball: String,
    pub shasum: String,
}
