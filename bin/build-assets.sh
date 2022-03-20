#!/usr/bin/env bash

# stop on first error
set -e

echo "Removing old client assets"
rm -Rf src/main/resources/assets
rm -f src/main/resources/assets.jar
rm -f src/main/resources/checksum

echo "Building the client code"
cd src/main/webapp
npm run build
cd ../../..

echo "Creating a zip file of the assets"
cd src/main/resources
zip -q -r assets.zip assets

echo "Creating the assets.zip checksum file"
shasum --algorithm 1 assets.zip > checksum

echo "Clearing the assets dir since it is no longer needed"
rm -Rf assets

echo "Building the jar"
cd ../../..

echo "Removing old release jars"
rm -f build/releases/*.jar

./gradlew build

cp build/releases/*.jar release

echo "Updating the documentation's mathlingua.jar"
mkdir -p documentation/.mlg
cp build/releases/*.jar documentation/.mlg/mathlingua.jar
