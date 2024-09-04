#!/bin/sh

set -ea

if [ -z "$1" ]; then
  echo "Usage: $0 <build-target> where build-target is optional and can be all|workspace|lb"
fi

projRoot=`git rev-parse --show-toplevel`

# get the device id
deviceName=$(xcrun devicectl list devices --hide-default-columns --columns "Name" --hide-headers | sed -n '2p')
echo running on $deviceName

if [ -z "$deviceName" ];then
  echo "No target device was found, make sure your ipad/iphone are on the same network"
  exit 1
fi


build_workspace(){
  echo "building workspace"
  cd "$projRoot"/libs/content/workspace-ffi/SwiftWorkspace 
  ./create_libs.sh       
}

build_lb_rs(){
  echo "building lb-rs"
  cd "$projRoot"/libs/lb/lb_external_interface 
  make swift_libs 
}

# build workspace 
case "$1" in
  "workspace")
   build_workspace
    ;;
  "lb-rs")
     build_lb_rs 
    ;;
  "all")
    build_workspace
    build_lb_rs
    ;;
  *)
       ;;
esac




# build ios app 
cd "$projRoot"/clients/apple
xcodebuild -workspace ./lockbook.xcworkspace -scheme "Lockbook (iOS)" -sdk iphoneos18.0 -configuration Debug -archivePath ./build/Lockbook-iOS.xcarchive archive 
appBundlePath=$(xcrun devicectl device install app --device "$deviceName" ./build/Lockbook-iOS.xcarchive/Products/Applications/Lockbook.app/ | grep "installationURL:" | sed 's/.*installationURL: //')

echo app bundle path: $appBundlePath

# run the app 
xcrun devicectl device process launch --console --device "$deviceName" $appBundlePath
