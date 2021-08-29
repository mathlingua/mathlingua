/*
 * Copyright 2019 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

import org.jetbrains.kotlin.config.KotlinCompilerVersion
import com.adarshr.gradle.testlogger.theme.ThemeType

plugins {
    // for improved test output
    id("com.adarshr.test-logger") version "1.7.0"
    id("com.diffplug.spotless") version "5.7.0"
    id("com.github.johnrengelman.shadow") version "7.0.0"
    id("org.jetbrains.kotlin.jvm") version "1.5.21"
    id("application")
    id("jacoco")
    kotlin("plugin.serialization") version "1.5.20"
}

group = "mathlingua"
version = "0.9.0"

application {
    mainClass.set("mathlingua.cli.MainKt")
}

java {
    sourceCompatibility = JavaVersion.VERSION_1_8
}

repositories {
    mavenCentral()
    maven {
        url = uri("https://oss.sonatype.org/content/repositories/snapshots")
        mavenContent {
            snapshotsOnly()
        }
    }
}

dependencies {
    implementation(kotlin("gradle-plugin", version = "1.5.21"))
    implementation("com.fifesoft:rsyntaxtextarea:3.0.3")
    implementation(kotlin("stdlib", KotlinCompilerVersion.VERSION))
    implementation("com.github.ajalt:clikt:2.6.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.4.2")
    implementation("org.jetbrains:markdown:0.2.4")
    implementation("io.javalin:javalin:3.13.10")
    implementation("org.slf4j:slf4j-nop:1.7.32")
    implementation("org.jetbrains.kotlinx:kotlinx-serialization-json:1.2.2")
    implementation("com.fasterxml.jackson.module:jackson-module-kotlin:2.12.4")
    testImplementation("com.willowtreeapps.assertk:assertk-jvm:0.19")
    testImplementation(kotlin("reflect", version = "1.5.0"))
    testImplementation("org.junit.jupiter:junit-jupiter-api:5.5.1")
    testImplementation("com.tylerthrailkill.helpers:pretty-print:2.0.2")
    testImplementation("com.beust:klaxon:5.4")
    testRuntimeOnly("org.junit.jupiter:junit-jupiter-engine:5.5.1")
}

tasks {
    test {
        useJUnitPlatform()
        finalizedBy(jacocoTestReport)
    }

    jar {
        manifest {
            attributes["Main-Class"] = application.mainClass
        }
    }

    check {
        dependsOn(jacocoTestCoverageVerification)
    }

    shadowJar {
        destinationDirectory.set(File("build", "releases"))
        archiveClassifier.set("")
    }
}

testlogger {
    theme = ThemeType.MOCHA
}

jacoco {
    toolVersion = "0.8.7"
}

spotless {
    kotlin {
        ktfmt("0.18").dropboxStyle()
    }
}
