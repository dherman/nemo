use crate::support::sandbox::{sandbox, DistroMetadata, NodeFixture, NpmFixture, YarnFixture};
use hamcrest2::assert_that;
use hamcrest2::prelude::*;
use test_support::matchers::execs;

use volta_core::error::ExitCode;

const BASIC_PACKAGE_JSON: &str = r#"{
  "name": "test-package"
}"#;
const PACKAGE_JSON_WITH_EMPTY_LINE: &str = r#"{
  "name": "test-package"
}
"#;
const PACKAGE_JSON_WITH_EXTENDS: &str = r#"{
  "name": "test-package",
  "volta": {
    "node": "8.9.10",
    "extends": "./basic.json"
  }
}"#;

fn package_json_with_pinned_node(node: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}"
  }}
}}"#,
        node
    )
}

fn package_json_with_pinned_node_npm(node: &str, npm: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "npm": "{}"
  }}
}}"#,
        node, npm
    )
}

fn package_json_with_pinned_node_yarn(node_version: &str, yarn_version: &str) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, yarn_version
    )
}

fn package_json_with_pinned_node_npm_yarn(
    node_version: &str,
    npm_version: &str,
    yarn_version: &str,
) -> String {
    format!(
        r#"{{
  "name": "test-package",
  "volta": {{
    "node": "{}",
    "npm": "{}",
    "yarn": "{}"
  }}
}}"#,
        node_version, npm_version, yarn_version
    )
}

const NODE_VERSION_INFO: &str = r#"[
{"version":"v10.99.1040","npm":"6.2.26","lts": "Dubnium","files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v9.27.6","npm":"5.6.17","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v8.9.10","npm":"5.6.7","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]},
{"version":"v6.19.62","npm":"3.10.1066","lts": false,"files":["linux-x64","osx-x64-tar","win-x64-zip","win-x86-zip", "linux-arm64"]}
]
"#;

cfg_if::cfg_if! {
    if #[cfg(target_os = "macos")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "linux")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 272,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 270,
                uncompressed_size: Some(0x0028_0000),
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 273,
                uncompressed_size: Some(0x0028_0000),
            },
        ];
    } else if #[cfg(target_os = "windows")] {
        const NODE_VERSION_FIXTURES: [DistroMetadata; 4] = [
            DistroMetadata {
                version: "10.99.1040",
                compressed_size: 1096,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "9.27.6",
                compressed_size: 1068,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "8.9.10",
                compressed_size: 1055,
                uncompressed_size: None,
            },
            DistroMetadata {
                version: "6.19.62",
                compressed_size: 1056,
                uncompressed_size: None,
            },
        ];
    } else {
        compile_error!("Unsupported target_os for tests (expected 'macos', 'linux', or 'windows').");
    }
}

const YARN_VERSION_INFO: &str = r#"{
    "name":"yarn",
    "dist-tags": { "latest":"1.12.99" },
    "versions": {
        "1.2.42": { "version":"1.2.42", "dist": { "shasum":"", "tarball":"" }},
        "1.4.159": { "version":"1.4.159", "dist": { "shasum":"", "tarball":"" }},
        "1.7.71": { "version":"1.7.71", "dist": { "shasum":"", "tarball":"" }},
        "1.12.99": { "version":"1.12.99", "dist": { "shasum":"", "tarball":"" }}
    }
}"#;

const YARN_VERSION_FIXTURES: [DistroMetadata; 4] = [
    DistroMetadata {
        version: "1.12.99",
        compressed_size: 178,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.7.71",
        compressed_size: 176,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.4.159",
        compressed_size: 177,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "1.2.42",
        compressed_size: 174,
        uncompressed_size: Some(0x0028_0000),
    },
];

const NPM_VERSION_FIXTURES: [DistroMetadata; 3] = [
    DistroMetadata {
        version: "1.2.3",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "4.5.6",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
    DistroMetadata {
        version: "8.1.5",
        compressed_size: 239,
        uncompressed_size: Some(0x0028_0000),
    },
];

const NPM_VERSION_INFO: &str = r#"
{
    "name":"npm",
    "dist-tags": { "latest":"8.1.5" },
    "versions": {
        "1.2.3": { "version":"1.2.3", "dist": { "shasum":"", "tarball":"" }},
        "4.5.6": { "version":"4.5.6", "dist": { "shasum":"", "tarball":"" }},
        "8.1.5": { "version":"8.1.5", "dist": { "shasum":"", "tarball":"" }}
    }
}
"#;

const VOLTA_LOGLEVEL: &str = "VOLTA_LOGLEVEL";

#[test]
fn pin_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("6.19.62"),
    )
}

#[test]
fn pin_node_reports_info() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]pinned node@6.19.62 (with npm@3.10.1066) in package.json")
    );
}

#[test]
fn pin_node_latest() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@latest"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.99.1040"),
    )
}

#[test]
fn pin_node_no_version() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("10.99.1040"),
    )
}

#[test]
fn pin_node_informs_newer_npm() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("8.9.10", "5.6.17"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.volta("pin node@10.99.1040"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]this version of Node includes npm@6.2.26, which is higher than your pinned version (5.6.17).")
            .with_stdout_contains("[..]`volta pin npm@bundled`[..]")
    );
}

#[test]
fn pin_node_with_npm_hides_bundled_version() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("8.9.10", "6.2.26"))
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.volta("pin node@9.27.6"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_does_not_contain("[..](with npm@5.6.17)[..]")
    );
}

#[test]
fn pin_yarn_no_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains(
                "[..]Cannot pin Yarn because the Node version is not pinned in this project."
            )
    );

    assert_eq!(s.read_package_json(), BASIC_PACKAGE_JSON)
}

#[test]
fn pin_yarn() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.4.159"),
    )
}

#[test]
fn pin_yarn_reports_info() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]pinned yarn@1.4.159 in package.json")
    );
}

#[test]
fn pin_yarn_latest() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@latest"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.12.99"),
    )
}

#[test]
fn pin_yarn_no_version() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("1.2.3", "1.12.99"),
    )
}

#[test]
fn pin_yarn_missing_release() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .mock_not_found()
        .build();

    assert_that!(
        s.volta("pin yarn@1.3.1"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download yarn@1.3.1")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("1.2.3"),
    )
}

#[test]
fn pin_yarn_leaves_npm() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "3.4.5"))
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm_yarn("1.2.3", "3.4.5", "1.4.159"),
    )
}

#[test]
fn pin_npm_no_node() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin npm@1.2.3"),
        execs()
            .with_status(ExitCode::ConfigurationError as i32)
            .with_stderr_contains(
                "[..]Cannot pin npm because the Node version is not pinned in this project."
            )
    );

    assert_eq!(s.read_package_json(), BASIC_PACKAGE_JSON)
}

#[test]
fn pin_npm() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin npm@4.5"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("1.2.3", "4.5.6"),
    )
}

#[test]
fn pin_npm_reports_info() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .env(VOLTA_LOGLEVEL, "info")
        .build();

    assert_that!(
        s.volta("pin npm@4.5"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]pinned npm@4.5.6 in package.json")
    );
}

#[test]
fn pin_npm_latest() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin npm@latest"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("1.2.3", "8.1.5"),
    );
}

#[test]
fn pin_npm_no_version() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .npm_available_versions(NPM_VERSION_INFO)
        .distro_mocks::<NpmFixture>(&NPM_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin npm"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_npm("1.2.3", "8.1.5"),
    )
}

#[test]
fn pin_npm_missing_release() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node("1.2.3"))
        .mock_not_found()
        .build();

    assert_that!(
        s.volta("pin npm@8.1.5"),
        execs()
            .with_status(ExitCode::NetworkError as i32)
            .with_stderr_contains("[..]Could not download npm@8.1.5")
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("1.2.3"),
    );
}

#[test]
fn pin_npm_bundled_removes_npm() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "4.5.6"))
        .node_npm_version_file("1.2.3", "3.2.1")
        .build();

    assert_that!(
        s.volta("pin npm@bundled"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node("1.2.3"),
    );
}

#[test]
fn pin_npm_bundled_reports_info() {
    let s = sandbox()
        .package_json(&package_json_with_pinned_node_npm("1.2.3", "4.5.6"))
        .node_npm_version_file("1.2.3", "3.2.1")
        .env("VOLTA_LOGLEVEL", "info")
        .build();

    assert_that!(
        s.volta("pin npm@bundled"),
        execs()
            .with_status(ExitCode::Success as i32)
            .with_stdout_contains("[..]set package.json to use bundled npm (currently 3.2.1)[..]")
    );
}

#[test]
fn pin_node_and_yarn() {
    let s = sandbox()
        .package_json(BASIC_PACKAGE_JSON)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .yarn_available_versions(YARN_VERSION_INFO)
        .distro_mocks::<YarnFixture>(&YARN_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@6 yarn@1.4"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert_eq!(
        s.read_package_json(),
        package_json_with_pinned_node_yarn("6.19.62", "1.4.159"),
    )
}

#[test]
fn pin_node_does_not_remove_trailing_newline() {
    let s = sandbox()
        .package_json(PACKAGE_JSON_WITH_EMPTY_LINE)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert!(s.read_package_json().ends_with('\n'))
}

#[test]
fn pin_node_does_not_overwrite_extends() {
    let s = sandbox()
        .package_json(PACKAGE_JSON_WITH_EXTENDS)
        .node_available_versions(NODE_VERSION_INFO)
        .distro_mocks::<NodeFixture>(&NODE_VERSION_FIXTURES)
        .project_file("basic.json", BASIC_PACKAGE_JSON)
        .build();

    assert_that!(
        s.volta("pin node@6"),
        execs().with_status(ExitCode::Success as i32)
    );

    assert!(s
        .read_package_json()
        .contains(r#""extends": "./basic.json""#));
}
