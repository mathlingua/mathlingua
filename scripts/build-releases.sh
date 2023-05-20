#!/usr/bin/env bash

# stop on first error
set -e

./scripts/build-artifacts.sh
./scripts/sign-artifacts.sh ${*}
