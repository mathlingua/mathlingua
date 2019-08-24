#!/usr/bin/env bash

kotlinc-js -output docs/bundle.js -meta-info $(find src/main/kotlin/mathlingua/common -name '*.kt')
