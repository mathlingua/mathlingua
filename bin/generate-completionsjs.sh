#!/usr/bin/env bash

./gradlew assemble
cp bin/header.txt docs/completions.js
java -cp build/libs/mathlingua-1.0-SNAPSHOT.jar mathlingua.jvm.HtmlCompletionsGenerator >> docs/completions.js
