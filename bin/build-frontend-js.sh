#!/usr/bin/env bash

echo "Generating the completions"
./bin/generate-completionsjs.sh

echo "Generating the data"
./bin/generate-datajs.sh

echo "Generating the bundle"
./bin/build-bundle.sh
