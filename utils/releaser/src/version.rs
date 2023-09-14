use crate::utils::{core_version, CommandRunner};
use clap::ValueEnum;
use regex::{Captures, Regex};
use std::fmt::{Display, Formatter};
use std::fs;
use std::process::Command;
use time::OffsetDateTime;
use toml_edit::{value, Document};

pub fn bump(bump_type: BumpType) {
    let new_version = determine_new_version(bump_type);

    ensure_clean_start_state();

    handle_cargo_tomls(&new_version);
    handle_apple(&new_version);
    handle_android(&new_version);
    generate_lockfile();

    push_to_git(&new_version);
}

#[derive(Copy, Clone, ValueEnum, PartialEq, Default, Debug)]
pub enum BumpType {
    Major,
    Minor,

    #[default]
    Patch,
}

impl Display for BumpType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{:?}", self).to_ascii_lowercase())
    }
}

fn handle_cargo_tomls(version: &str) {
    let cargos_to_update = vec![
        "clients/admin",
        "clients/cli",
        "clients/egui",
        "server/server",
        "libs/core",
        "libs/core/libs/shared",
        "libs/core/libs/test_utils",
        "libs/content/editor/egui_editor",
        "libs/core/c_interface_v2",
        "libs/core/core_external_interface",
        "utils/dev-tool",
        "utils/releaser",
        "utils/winstaller",
    ];

    for cargo_path in cargos_to_update {
        let cargo_path = &[cargo_path, "/Cargo.toml"].join("");
        let mut cargo_toml = fs::read_to_string(cargo_path)
            .unwrap()
            .parse::<Document>()
            .unwrap();

        cargo_toml["package"]["version"] = value(version);
        fs::write(cargo_path, cargo_toml.to_string()).unwrap();
    }
}

fn handle_apple(version: &str) {
    let plists = ["clients/apple/iOS/info.plist", "clients/apple/macOS/info.plist"];
    for plist in plists {
        Command::new("/usr/libexec/Plistbuddy")
            .args(["-c", &format!("Set CFBundleShortVersionString {version}"), plist])
            .assert_success();
        let now = OffsetDateTime::now_utc();
        let month = now.month() as u8;
        let day = now.day();
        let year = now.year();
        Command::new("/usr/libexec/Plistbuddy")
            .args(["-c", &format!("Set CFBundleVersion {year}{month}{day}"), plist])
            .assert_success();
    }
}

fn handle_android(version: &str) {
    let path = "clients/android/app/build.gradle";
    let mut gradle_build = fs::read_to_string(path).unwrap();

    let version_name_re = Regex::new(r"(versionName) (.*)").unwrap();
    let version_code_re = Regex::new(r"(versionCode) *(?P<version_code>\d+)").unwrap();
    let mut version_code = 0;
    for caps in version_code_re.captures_iter(&gradle_build) {
        version_code = caps["version_code"].parse().unwrap();
    }
    gradle_build = version_code_re
        .replace(&gradle_build, |caps: &Captures| format!("{} {}", &caps[1], version_code + 1))
        .to_string();

    gradle_build = version_name_re
        .replace(&gradle_build, |caps: &Captures| format!("{} \"{}\"", &caps[1], version))
        .to_string();

    fs::write(path, gradle_build).unwrap();
}

fn determine_new_version(bump_type: BumpType) -> String {
    let mut current_version: Vec<i32> = core_version()
        .split('.')
        .map(|f| f.parse().unwrap())
        .collect();

    match bump_type {
        BumpType::Major => {
            current_version[0] += 1;
            current_version[1] = 0;
            current_version[2] = 0;
        }
        BumpType::Minor => {
            current_version[1] += 1;
            current_version[2] = 0;
        }
        BumpType::Patch => current_version[2] += 1,
    }

    current_version
        .iter()
        .map(|f| f.to_string())
        .collect::<Vec<String>>()
        .join(".")
}

fn generate_lockfile() {
    Command::new("cargo").arg("check").assert_success();
}

fn ensure_clean_start_state() {
    Command::new("git")
        .args(["diff", "--exit-code"])
        .assert_success()
}

fn push_to_git(version: &str) {
    Command::new("bash")
        .args([
            "-c",
            &format!("git checkout -b bump-{version} && git add -A && git commit -m 'bump-{version}' && git push origin bump-{version}")
        ])
        .assert_success()
}
