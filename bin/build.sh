#!/usr/bin/env bash

# stop on first error
set -e

echo "Removing old client assets"
rm -Rf src/main/resources/assets

echo "Building the client code"
cd src/main/webapp
npm run build

echo "Building the jar"
cd ../../..
./gradlew build

