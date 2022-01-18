#!/usr/bin/env bash

# stop on first error
set -e

MLG_VERSION="0.15.0"

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

./bin/build-assets.sh

echo "Updating the documentation's mlg"
mkdir -p documentation
cp release/mlg-${MLG_VERSION}-darwin-arm64 documentation/mlg

if command -v codesign &> /dev/null
then
  echo "Signing the binary documentation/mlg"
  codesign -s - documentation/mlg

  for file in $(ls release/mlg-*-darwin-*)
  do
    echo "Signing the binary ${file}"
    codesign -s - ${file}
  done
else
  echo "WARNING: Could not sign the binaries since 'codesign' does not exist."
  echo "         Build the binaries on a MacOS system to be able to sign the binaries."
fi

echo "Building the documentation"
./bin/build-docs.sh
