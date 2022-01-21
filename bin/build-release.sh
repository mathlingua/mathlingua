#!/usr/bin/env bash

# stop on first error
set -e

MLG_VERSION="0.15.1"

echo "Building release ${MLG_VERSION}"

RED_ERROR="\033[0;31mERROR\033[0m"

GRADLE_VERSION_FILE=build.gradle.kts
echo "Checking version declaration in ${GRADLE_VERSION_FILE}"
if [[ "$(grep "version = \"${MLG_VERSION}\"" "${GRADLE_VERSION_FILE}")" == "" ]]
then
  echo -e "${RED_ERROR}: ${GRADLE_VERSION_FILE} doesn't have the version set to ${MLG_VERSION}"
  exit 1
fi

KOTLIN_VERSION_FILE=src/main/kotlin/mathlingua/cli/Mathlingua.kt
echo "Checking version declaration in ${KOTLIN_VERSION_FILE}"
if [[ "$(grep "const val MATHLINGUA_VERSION = \"${MLG_VERSION}\"" "${KOTLIN_VERSION_FILE}")" == "" ]]
then
  echo -e "${RED_ERROR}: ${KOTLIN_VERSION_FILE} doesn't have the version set to ${MLG_VERSION}"
  exit 1
fi

GO_VERSION_FILE=mlg/Main.go
echo "Checking version declaration in ${GO_VERSION_FILE}"
if [[ "$(grep "const MATHLINGUA_VERSION = \"${MLG_VERSION}\"" "${GO_VERSION_FILE}")" == "" ]]
then
  echo -e "${RED_ERROR}: ${GO_VERSION_FILE} doesn't have the version set to ${MLG_VERSION}"
  exit 1
fi

echo "Removing old release"
rm -Rf release
mkdir -p release

echo "Building the launcher"
cd mlg

for OS in darwin linux windows
do
  for ARCH in 386 amd64 arm arm64
  do
    if [[ "${OS}" != "darwin" ]] || [[ "${ARCH}" == "amd64" ]] || [[ "${ARCH}"  == "arm64" ]]
    then
      EXT=""
      if [[ "${OS}" == "windows" ]]
      then
        EXT=".exe"
      fi

      echo "Building for ${OS} ${ARCH}"
      env GOOS="${OS}" GOARCH="${ARCH}" go build -o ../release/mlg-${MLG_VERSION}-${OS}-${ARCH}${EXT}
    fi
  done
done

cd ..

echo "Making all of the built binaries executable"
chmod +x release/*

./bin/build-assets.sh

# sign the MacOS binaries if possible and notarize them if
# the --notarize option is specified
if command -v codesign &> /dev/null
then
  if [ -z "$MLG_DEV_APPLE_CERTIFICATE_ID" ]; then
    echo "You must set the MLG_DEV_APPLE_CERTIFICATE_ID environment variable"
    echo "to be Apple Developer Certificate ID to use for signing and notarizing."
    echo "See https://artyom.dev/notarizing-go-binaries-for-macos.html for details"
    echo "on how to obtain this ID."
    exit 1
  fi

  if [ -z "$MLG_DEV_APPLE_EMAIL" ]; then
    echo "You must set the MLG_DEV_APPLE_EMAIL environment variable to be the"
    echo "email of the Apple Developer account to use for signing and notarizing."
    echo "See https://artyom.dev/notarizing-go-binaries-for-macos.html for details"
    echo "about the notarization process."
    exit 1
  fi

  if [ "$1" = "--notarize" ]; then
    echo "Attempting to notarize the MacOS binaries"
    for file in $(ls release/mlg-*-darwin-*)
    do
      echo "Signing the binary ${file} with the certificate in \$MLG_DEV_APPLE_CERTIFICATE_ID"
      codesign --timestamp -s "$MLG_DEV_APPLE_CERTIFICATE_ID" -o runtime -v "${file}"
    done

    MAC_OS_BINARIES_ZIP=release/macos-binaries.zip
    echo "Bundling the MacOS binaries into ${MAC_OS_BINARIES_ZIP}"
    zip -j "${MAC_OS_BINARIES_ZIP}" $(ls release/mlg-*-darwin-*)

    echo "Sending the MacOS binaries for notarization"
    xcrun altool --notarize-app \
                 --primary-bundle-id com.github.mathlingua \
                 --username "${MLG_DEV_APPLE_EMAIL}" \
                 --password '@keychain:altool' \
                 --file "${MAC_OS_BINARIES_ZIP}"

    echo "Use the following command to check the status of the notarization"
    echo "process where REQUEST-UUID can be obtained from the output above."
    echo "You should also receive an email when the notarization completes"
    echo "or encounters an error."
    echo ""
    echo "xcrun altool --notarization-info <REQUEST-UUID> \\"
    echo "             -u ${MLG_DEV_APPLE_EMAIL} \\"
    echo "             -p \"@keychain:altool\""
  else
    for file in $(ls release/mlg-*-darwin-*)
    do
      echo "Ad-hoc signing the binary ${file}"
      codesign -s - ${file}
    done
  fi
else
  echo "WARNING: Could not sign the binaries since 'codesign' does not exist."
  echo "         Build the binaries on a MacOS system to be able to sign the binaries."
fi

echo "Updating the documentation's mlg"
mkdir -p documentation
cp release/mlg-${MLG_VERSION}-darwin-arm64 documentation/mlg

echo "Building the documentation"
./bin/build-docs.sh
