#!/usr/bin/env bash

./gradlew assemble
cp bin/header.txt docs/data.js
java -cp build/libs/mathlingua-1.0-SNAPSHOT.jar mathlingua.jvm.HtmlDataGenerator >> docs/data.js

