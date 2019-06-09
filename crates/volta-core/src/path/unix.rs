//! Provides functions for determining the paths of files and directories
//! in a standard Volta layout in Unix-based operating systems.

use std::io;
use std::os::unix;
use std::path::{Path, PathBuf};

use dirs;

use crate::distro::node::NodeDistro;
use crate::error::ErrorDetails;
use volta_fail::Fallible;

use super::{node_image_dir, shim_dir, volta_home};

// ~/
//     .volta/
//         cache/                                          cache_dir
//             node/                                       node_cache_dir
//                 index.json                              node_index_file
//                 index.json.expires                      node_index_expiry_file
//         bin/                                            shim_dir
//             node                                        shim_file("node")
//             yarn
//             npm
//             npx
//             ...
//             ember
//         log/                                            log_dir
//         tools/                                          tools_dir
//             inventory/                                  inventory_dir
//                 node/                                   node_inventory_dir
//                     node-v4.8.4-linux-x64.tar.gz        node_distro_file_name("4.8.4")
//                     node-v4.8.4-npm                     node_npm_version_file("4.8.4")
//                     ...
//                 packages/                               package_inventory_dir
//                     ember-cli-3.7.1.tgz                 package_distro_file("ember-cli", "3.7.1")
//                     ember-cli-3.7.1.shasum              package_distro_shasum("ember-cli", "3.7.1")
//                 yarn/                                   yarn_inventory_dir
//             image/                                      image_dir
//                 node/                                   node_image_root_dir
//                     10.13.0/
//                         6.4.0/                          node_image_dir("10.13.0", "6.4.0")
//                             bin/                        node_image_bin_dir("10.13.0", "6.4.0")
//                 yarn/                                   yarn_image_root_dir
//                     1.7.0/                              yarn_image_dir("1.7.0")
//                 packages/                               package_image_root_dir
//                     ember-cli/
//                         3.7.1/                          package_image_dir("ember-cli", "3.7.1")
//             user/                                       user_toolchain_dir
//                 bins/
//                     tsc.json                            user_tool_bin_config("tsc")
//                 packages/                               user_package_dir
//                     ember-cli.json                      user_package_config_file("ember-cli")
//                 platform.json                           user_platform_file
//         volta                                           volta_file
//         shim                                            shim_executable
//         hooks.json                                      user_hooks_file

pub fn default_volta_home() -> Fallible<PathBuf> {
    let home = dirs::home_dir().ok_or(ErrorDetails::NoHomeEnvironmentVar)?;
    Ok(home.join(".volta"))
}

pub fn archive_extension() -> String {
    String::from("tar.gz")
}

pub fn node_image_bin_dir(node: &str, npm: &str) -> Fallible<PathBuf> {
    Ok(node_image_dir(node, npm)?.join("bin"))
}

pub fn node_archive_npm_package_json_path(version: &str) -> PathBuf {
    Path::new(&NodeDistro::basename(version))
        .join("lib")
        .join("node_modules")
        .join("npm")
        .join("package.json")
}

pub fn shim_file(toolname: &str) -> Fallible<PathBuf> {
    Ok(shim_dir()?.join(toolname))
}

pub fn volta_file() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("volta"))
}

pub fn shim_executable() -> Fallible<PathBuf> {
    Ok(volta_home()?.join("shim"))
}

pub fn env_paths() -> Fallible<Vec<PathBuf>> {
    Ok(vec![shim_dir()?])
}

/// Create a symlink. The `dst` path will be a symbolic link pointing to the `src` path.
pub fn create_file_symlink(src: PathBuf, dst: PathBuf) -> Result<(), io::Error> {
    unix::fs::symlink(src, dst)
}
