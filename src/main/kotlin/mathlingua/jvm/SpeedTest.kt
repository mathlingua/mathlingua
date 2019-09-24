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

package mathlingua.jvm

import mathlingua.common.MathLingua
import java.nio.file.Paths

fun main(args: Array<String>) {
    println("Starting the speed test")

    val ml = MathLingua()
    val path = Paths.get(args[0])

    val startRead = System.currentTimeMillis()
    val text = path.toFile().readText()
    val endRead = System.currentTimeMillis()

    println("Read the text in ${endRead - startRead} ms")

    val startParse = System.currentTimeMillis()
    ml.parse(text)
    val endParse = System.currentTimeMillis()

    println("Parsed the text in ${endParse - startParse} ms")
}

/*

50,000
Starting the speed test
Read the text in 10 ms
Parsed the text in 1367 ms

100,000
Starting the speed test
Read the text in 19 ms
Parsed the text in 2077 ms

200,000
Starting the speed test
Read the text in 22 ms
Parsed the text in 3569 ms

300,000
Starting the speed test
Read the text in 21 ms
Parsed the text in 4482 ms

400,000
Starting the speed test
Read the text in 25 ms
Parsed the text in 5441 ms

~2.4 million
Starting the speed test
Read the text in 160 ms
Parsed the text in 27016 ms

linear regression:
y = 0.01164*x + 943.28659

 */