#!/usr/bin/env bash

# stop on first error
set -e

./gradlew build

cd src/main/webapp/cypress-codex
java -jar ../../../../build/releases/mathlingua-0.10.1.jar edit
