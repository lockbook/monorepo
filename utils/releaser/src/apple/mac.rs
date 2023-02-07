use crate::secrets::AppStore;
use crate::utils::{core_version, lb_repo, CommandRunner};
use crate::Github;
use gh_release::ReleaseClient;
use std::fs::File;
use std::process::Command;

pub fn release(asc: &AppStore, gh: &Github) {
    archive();
    notarize(asc);
    upload_gh(gh);
    upload_app_store(asc);
}

fn archive() {
    Command::new("xcodebuild")
        .args([
            "-workspace",
            "clients/apple/Lockbook.xcworkspace",
            "-scheme",
            "Lockbook (macOS)",
            "-sdk",
            "macosx",
            "-configuration",
            "Release",
            "-archivePath",
            "clients/apple/build/Lockbook-macOS.xcarchive",
            "archive",
        ])
        .assert_success();

    // creates .app to upload to github
    Command::new("xcodebuild")
        .args([
            "-allowProvisioningUpdates",
            "-archivePath",
            "clients/apple/build/Lockbook-macOS.xcarchive",
            "-exportPath",
            "clients/apple/build/",
            "-exportOptionsPlist",
            "clients/apple/exportOptionsGHApp.plist",
            "-exportArchive",
        ])
        .assert_success();

    // creates .pkg to upload to the app store
    Command::new("xcodebuild")
        .args([
            "-allowProvisioningUpdates",
            "-archivePath",
            "clients/apple/build/Lockbook-macOS.xcarchive",
            "-exportPath",
            "clients/apple/build/",
            "-exportOptionsPlist",
            "clients/apple/exportOptions.plist",
            "-exportArchive",
        ])
        .assert_success();
}

fn notarize(asc: &AppStore) {
    Command::new("ditto")
        .arg("-c")
        .arg("-k")
        .arg("--keepParent")
        .arg("Lockbook.app")
        .arg("lockbook-macos.app.zip")
        .current_dir("clients/apple/build")
        .assert_success();

    Command::new("xcrun")
        .args([
            "notarytool",
            "submit",
            "clients/apple/build/lockbook-macos.app.zip",
            "--apple-id",
            "parth@mehrotra.me",
            "--password",
            &asc.0,
            "--team-id",
            "39ZS78S25U",
            "--wait",
        ])
        .assert_success();

    Command::new("xcrun")
        .args(["stapler", "staple", "-v", "clients/apple/build/Lockbook.app"])
        .assert_success();

    Command::new("ditto")
        .arg("-c")
        .arg("-k")
        .arg("--keepParent")
        .arg("Lockbook.app")
        .arg("lockbook-macos.app.zip")
        .current_dir("clients/apple/build")
        .assert_success();
}

fn upload_gh(gh: &Github) {
    let client = ReleaseClient::new(gh.0.clone()).unwrap();
    let release = client
        .get_release_by_tag_name(&lb_repo(), &core_version())
        .unwrap();
    let file = File::open("clients/apple/build/lockbook-macos.app.zip").unwrap();
    client
        .upload_release_asset(
            &lb_repo(),
            release.id,
            "lockbook-macos.app.zip",
            "application/zip",
            file,
            None,
        )
        .unwrap();
}

fn upload_app_store(asc: &AppStore) {
    Command::new("xcrun")
        .args([
            "altool",
            "--upload-app",
            "-t",
            "macos",
            "-f",
            "clients/apple/build/Lockbook.pkg",
            "-u",
            "parth@mehrotra.me",
            "-p",
            &asc.0,
        ])
        .assert_success();
}
