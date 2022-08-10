use crate::utils::HashInfo;
use crate::{panic_if_unsuccessful, utils, ToolEnvironment};
use std::fs;
use std::fs::File;
use std::io::Write;
use std::process::{Command, Stdio};

const LIB_NAME_HEADER: &str = "lockbook_core.h";
const LIB_NAME: &str = "liblockbook_core.a";

pub fn run_swift_tests(tool_env: ToolEnvironment) {
    let hash_info = HashInfo::get_from_dir(&tool_env.hash_info_dir, &tool_env.commit_hash);
    dotenv::from_path(utils::test_env_path(&tool_env.root_dir)).unwrap();

    make_swift_test_lib(tool_env.clone());

    let build_results = Command::new("swift")
        .arg("build")
        .current_dir(utils::swift_dir(&tool_env.root_dir))
        .status()
        .unwrap();

    panic_if_unsuccessful!(build_results);

    let test_results = Command::new("swift")
        .args(&["test", "--generate-linuxmain"])
        .current_dir(utils::swift_dir(&tool_env.root_dir))
        .env("API_URL", utils::get_api_url(hash_info.get_port()))
        .status()
        .unwrap();

    panic_if_unsuccessful!(test_results)
}

pub fn make_swift_test_lib(tool_env: ToolEnvironment) {
    let core_dir = utils::core_dir(&tool_env.root_dir);
    let c_interface_dir = core_dir
        .join("src/external_interface/c_interface.rs")
        .to_str()
        .unwrap()
        .to_string();

    let build_results = Command::new("cbindgen")
        .args([&c_interface_dir, "-l", "c"])
        .current_dir(utils::core_dir(&tool_env.root_dir))
        .stdout(Stdio::piped())
        .output()
        .unwrap();

    panic_if_unsuccessful!(build_results.status);

    let swift_inc_dir = utils::swift_inc(&tool_env.root_dir);

    fs::create_dir_all(&swift_inc_dir).unwrap();
    File::create(swift_inc_dir.join(LIB_NAME_HEADER))
        .unwrap()
        .write_all(build_results.stdout.as_slice())
        .unwrap();

    let build_results = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(utils::core_dir(&tool_env.root_dir))
        .status()
        .unwrap();

    panic_if_unsuccessful!(build_results);

    let swift_lib_dir = utils::swift_lib(&tool_env.root_dir);

    fs::create_dir_all(&swift_lib_dir).unwrap();

    fs::copy(tool_env.target_dir.join("release").join(LIB_NAME), swift_lib_dir.join(LIB_NAME))
        .unwrap();
}
