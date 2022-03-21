#!/usr/bin/env bash

./bin/build-release.sh

JAR_FILE=$(ls build/releases/mathlingua-*.jar)

mkdir -p release

$GRAALVM_HOME/bin/native-image \
  --no-fallback \
  --enable-all-security-services \
  --report-unsupported-elements-at-runtime \
  --install-exit-handlers \
  --allow-incomplete-classpath \
  --initialize-at-build-time=io.ktor,kotlinx,kotlin,org.slf4j,ch.qos.logback.classic.Logger,ch.qos.logback.classic.Level,ch.qos.logback.core.status.InfoStatus,ch.qos.logback.core.spi.AppenderAttachableImpl \
  -H:+ReportUnsupportedElementsAtRuntime \
  -H:+ReportExceptionStackTraces \
  -H:ReflectionConfigurationFiles=./reflection.json \
  -H:IncludeResources=assets.zip \
  -H:IncludeResources=checksum \
  -cp "${JAR_FILE}" \
  -H:Class=mathlingua.cli.MainKt \
  -H:Name=release/mlg
