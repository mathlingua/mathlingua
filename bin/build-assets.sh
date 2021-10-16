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

cp build/releases/*.jar release

echo "Updating the documentation's mathlingua.jar"
mkdir -p documentation/.bin
cp build/releases/*.jar documentation/.bin/mathlingua.jar
