use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{Context, ErrorKind, Fallible};
use crate::fs::read_dir_eager;
use crate::version::{hashmap_version_serde, version_serde};
use cfg_if::cfg_if;
use semver::Version;
use serde::Deserialize;

// Accept header needed to request the abbreviated metadata from the npm registry
// See https://github.com/npm/registry/blob/master/docs/responses/package-metadata.md
pub const NPM_ABBREVIATED_ACCEPT_HEADER: &str =
    "application/vnd.npm.install-v1+json; q=1.0, application/json; q=0.8, */*";

cfg_if! {
    if #[cfg(feature = "mock-network")] {
        pub fn public_registry_index(package: &str) -> String {
            format!("{}/{}", mockito::SERVER_URL, package)
        }
    } else {
        pub fn public_registry_index(package: &str) -> String {
            format!("https://registry.npmjs.org/{}", package)
        }
    }
}

pub fn public_registry_package(package: &str, version: &str) -> String {
    format!(
        "{}/-/{}-{}.tgz",
        public_registry_index(package),
        package,
        version
    )
}

/// Figure out the unpacked package directory name dynamically
///
/// Packages typically extract to a "package" directory, but not always
pub fn find_unpack_dir(in_dir: &Path) -> Fallible<PathBuf> {
    let dirs: Vec<_> = read_dir_eager(in_dir)
        .with_context(|| ErrorKind::PackageUnpackError)?
        .collect();

    // if there is only one directory, return that
    if let [(entry, metadata)] = dirs.as_slice() {
        if metadata.is_dir() {
            return Ok(entry.path());
        }
    }
    // there is more than just a single directory here, something is wrong
    Err(ErrorKind::PackageUnpackError.into())
}

/// Details about a package in the npm Registry
#[derive(Debug)]
pub struct PackageDetails {
    pub(crate) version: Version,
    pub(crate) tarball_url: String,
    pub(crate) shasum: String,
}

/// Index of versions of a specific package from the npm Registry
pub struct PackageIndex {
    pub tags: HashMap<String, Version>,
    pub entries: Vec<PackageDetails>,
}

/// Package Metadata Response
///
/// See npm registry API doc:
/// https://github.com/npm/registry/blob/master/docs/REGISTRY-API.md
#[derive(Deserialize, Debug)]
pub struct RawPackageMetadata {
    pub name: String,
    pub versions: HashMap<String, RawPackageVersionInfo>,
    #[serde(
        rename = "dist-tags",
        deserialize_with = "hashmap_version_serde::deserialize"
    )]
    pub dist_tags: HashMap<String, Version>,
}

#[derive(Deserialize, Debug)]
pub struct RawPackageVersionInfo {
    // there's a lot more in there, but right now just care about the version
    #[serde(with = "version_serde")]
    pub version: Version,
    pub dist: RawDistInfo,
}

#[derive(Deserialize, Clone, Debug)]
pub struct RawDistInfo {
    pub shasum: String,
    pub tarball: String,
}

impl From<RawPackageMetadata> for PackageIndex {
    fn from(serial: RawPackageMetadata) -> PackageIndex {
        let mut entries: Vec<PackageDetails> = serial
            .versions
            .into_iter()
            .map(|(_, version_info)| PackageDetails {
                version: version_info.version,
                tarball_url: version_info.dist.tarball,
                shasum: version_info.dist.shasum,
            })
            .collect();

        entries.sort_by(|a, b| b.version.cmp(&a.version));

        PackageIndex {
            tags: serial.dist_tags,
            entries,
        }
    }
}
