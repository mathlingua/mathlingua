#!/usr/bin/env bash

# stop on first error
set -e

RELEASE_DIR=release

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
      env GOOS="${OS}" GOARCH="${ARCH}" go build -o ${RELEASE_DIR}/mlg-${MLG_VERSION}-${OS}-${ARCH}${EXT}
    fi
  done
done
