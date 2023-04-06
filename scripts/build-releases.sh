#!/usr/bin/env bash

# stop on first error
set -e

VERSION=$(cat pkg/mlg/mlg.go | \
          grep -A1 'func (m \*Mlg) Version() string' | \
          grep 'return' | \
          sed 's|\treturn "v||' | \
          sed 's|"||')

if [ -z "$VERSION" ]
then
  echo "ERROR: Could not determine the version"
  exit
else
  echo "Using version $VERSION"
fi

RELEASE_DIR=release

echo "Cleaning previous builds"
make clean

echo "Building the web code"
make web

echo "Initializing release directory"
rm -Rf ${RELEASE_DIR}
mkdir -p ${RELEASE_DIR}

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
      env GOOS="${OS}" GOARCH="${ARCH}" go build -o ${RELEASE_DIR}/mlg-${VERSION}-${OS}-${ARCH}${EXT}
    fi
  done
done


# sign the MacOS binaries if possible and notarize them if
# the --notarize option is specified
if command -v codesign &> /dev/null
then
  if [ -z "$MLG_DEV_APPLE_CERTIFICATE_ID" ]; then
    echo "You must set the MLG_DEV_APPLE_CERTIFICATE_ID environment variable"
    echo "to be Apple Developer Certificate ID to use for signing and notarizing."
    echo "See https://artyom.dev/notarizing-go-binaries-for-macos.md for details"
    echo "on how to obtain this ID."
    exit 1
  fi

  if [ -z "$MLG_DEV_APPLE_EMAIL" ]; then
    echo "You must set the MLG_DEV_APPLE_EMAIL environment variable to be the"
    echo "email of the Apple Developer account to use for signing and notarizing."
    echo "See https://artyom.dev/notarizing-go-binaries-for-macos.md for details"
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
