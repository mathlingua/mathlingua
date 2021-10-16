#!/usr/bin/env bash

# stop on first error
set -e

# remove the old docs directory
rm -Rf docs

cd documentation
./mlg document
mv docs ..
cd ..

