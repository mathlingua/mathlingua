#!/usr/bin/env bash

# stop on first error
set -e

VERSION=$(cat pkg/mlg/mlg.go | \
          grep -A1 'func (m \*mlg) Version() string' | \
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
