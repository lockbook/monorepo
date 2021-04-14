#!/bin/sh
set -ae

if [ -z "$LOCKBOOK_CLI_PPA_LOCATION" ]
then
	echo "No LOCKBOOK_CLI_PPA_LOCATION, we need debian package files to build from"
	exit 69
fi

if [ -z "$GITHUB_TOKEN" ]
then
	echo "No GITHUB_TOKEN, you won't be able to upload to github without this"
	exit 69
fi

if ! command -v github-release &> /dev/null
then
	echo "You do not have the util github-release, checkout https://github.com/github-release/github-release"
	exit 69
fi

current_branch=$(git rev-parse --abbrev-ref HEAD)
current_hash=$(git rev-parse --short HEAD)

if [ $current_branch != "master" ]
then
	echo "Do not release non-master code."
	exit 69
fi

if ! command -v debuild &> /dev/null
then
	echo "You do not have the util debuild, this is used to compile the package"
	exit 69
fi

if ! command -v dh &> /dev/null
then
	echo "You do not have the util debhelper, this is used to compile the package"
	exit 69
fi

if ! command -v equivs-build &> /dev/null
then
	echo "You do not have the util equivs, this is used to build the source package"
	exit 69
fi


cd $LOCKBOOK_CLI_PPA_LOCATION

current_version=$(dpkg-parsechangelog --show-field Version)

echo "Installing build dependencies"
mk-build-deps
apt install ./"lockbook-build-deps_${current_version}_all.deb"

echo "Setting up clean environment"
debuild -- clean

echo "Clean"
rm -f "lockbook-build-deps_${current_version}_all.deb"

echo "Compiling package"
debuild 

cd ..

sha_description=$(shasum -a 256 "lockbook_${current_version}_amd64.deb")
sha=$(echo $sha_description | cut -d ' ' -f 1)

echo "Releasing..."
github-release release \
	--user lockbook \
	--repo lockbook \
	--tag "debian-cli-$current_version" \
	--name "Lockbook CLI Debian" \
	--description "A debian package to easily install lockbook CLI." \
	--pre-release || echo "Failed to create release, perhaps because one exists, attempting upload"

github-release upload \
	--user lockbook \
	--repo lockbook \
	--tag "debian-cli-$current_version" \
	--name "lockbook_${current_version}_amd64.deb" \
	--file "lockbook_${current_version}_amd64.deb"

echo $sha_description >> DEBIAN_CLI_SHA256

github-release upload \
	--user lockbook \
	--repo lockbook \
	--tag "debian-cli-$current_version" \
	--name "debian-cli-sha256-$sha" \
	--file DEBIAN_CLI_SHA256

echo "Cleaning up"
rm -f "lockbook_${current_version}_amd64.build" \
	"lockbook_${current_version}_amd64.buildinfo" \
	"lockbook_${current_version}_amd64.changes" \
	"lockbook_${current_version}_amd64.deb"

echo "Verify this sha is a part of the release on github: $sha"
