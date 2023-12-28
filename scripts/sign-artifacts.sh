#!/usr/bin/env bash

# stop on first error
set -e

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

  if [ -z "$MLG_DEV_APP_PASSWORD" ]; then
    echo "You must set the MLG_DEV_APP_PASSWORD environment variable to an "
    echo "app-specific password.  See the 'How to generate an app-specific password'"
    echo "section of https://support.apple.com/en-us/HT204397."
    exit 1
  fi

  echo "Ensuring the mlg app-specific password is stored in the keychain"
  xcrun altool --store-password-in-keychain-item altool \
    -u "$MLG_DEV_APPLE_EMAIL" \
    -p "$MLG_DEV_APP_PASSWORD"

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
