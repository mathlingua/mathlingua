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
rm -f target/*.jar

mvn package

OUT_NAME=$(ls target/mathlingua-*-jar-with-dependencies.jar | sed 's|target/||' | sed 's|-jar-with-dependencies||')
mkdir -p release
cp target/mathlingua-*-jar-with-dependencies.jar release/${OUT_NAME}

echo "Updating the documentation's mathlingua.jar"
mkdir -p documentation/.mlg
cp release/${OUT_NAME} documentation/.mlg/mathlingua.jar
