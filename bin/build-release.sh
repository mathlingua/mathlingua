#!/usr/bin/env bash

# stop on first error
set -e

MLG_VERSION="0.11.0"

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
cp release/mlg-${MLG_VERSION}-darwin-amd64 documentation/mlg

echo "Building the documentation"
./bin/build-docs.sh
