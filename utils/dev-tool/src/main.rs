use std::path::PathBuf;
use structopt::StructOpt;

pub mod android;
pub mod apple;
pub mod server;
pub mod setup;
pub mod utils;
pub mod workspace;

#[derive(Debug, PartialEq, StructOpt)]
#[structopt(about = "Lockbook's development and ci tool.")]
enum Commands {
    /// Verify CI environment
    VerifyCIEnvironment,

    /// Check the formatting
    FmtCheck,

    /// Check the lint
    ClippyCheck,

    /// Check the formatting of the android client
    AndroidFmtCheck,

    /// Check the lint of the android client
    AndroidLintCheck,

    /// Make kotlin jni libs
    MakeKotlinLibs,

    /// Make kotlin jni libs for tests
    MakeKotlinTestLib,

    /// Make swift jni libs for tests
    MakeSwiftTestLib,

    /// Build server
    BuildServer,

    /// Run server detached
    RunServer,

    /// Run all rust tests
    RunRustTests,

    /// Run kotlin integration tests
    RunKotlinTests,

    /// Run swift integration tests
    RunSwiftTests,

    /// Kill server for commit hash
    KillServer,

    /// Kill all servers running
    KillAllServers,
}

#[derive(Clone)]
pub struct ToolEnvironment {
    root_dir: PathBuf,
    target_dir: PathBuf,
    hash_info_dir: PathBuf,
    commit_hash: String,
}

impl ToolEnvironment {
    pub fn new() -> ToolEnvironment {
        let (root_dir, target_dir, hash_info_dir) = utils::get_dirs();

        ToolEnvironment {
            root_dir,
            target_dir,
            hash_info_dir,
            commit_hash: utils::get_commit_hash(),
        }
    }
}

fn main() {
    let tool_env = ToolEnvironment::new();

    use Commands::*;
    match Commands::from_args() {
        VerifyCIEnvironment => setup::verify_ci_environment(),
        FmtCheck => workspace::fmt_workspace(tool_env),
        ClippyCheck => workspace::clippy_workspace(tool_env),
        AndroidFmtCheck => android::fmt_android(tool_env),
        AndroidLintCheck => android::lint_android(tool_env),
        MakeKotlinLibs => android::make_android_libs(tool_env),
        MakeKotlinTestLib => android::make_android_test_lib(tool_env),
        MakeSwiftTestLib => apple::make_swift_test_lib(tool_env),
        BuildServer => server::build_server(tool_env),
        RunServer => server::run_server_detached(tool_env),
        RunRustTests => server::run_rust_tests(tool_env),
        RunKotlinTests => android::run_kotlin_tests(tool_env),
        RunSwiftTests => apple::run_swift_tests(tool_env),
        KillServer => server::kill_server(tool_env),
        KillAllServers => server::kill_all_servers(tool_env),
    }
}
