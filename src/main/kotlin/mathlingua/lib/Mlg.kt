/*
 * Copyright 2022 Dominic Kramer
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

package mathlingua.lib

import java.io.File

interface Mlg {
    fun check(files: List<File>)
    fun edit(noOpen: Boolean, port: Int)
    fun doc()
    fun clean()
    fun version()
}

fun newMlg(): Mlg = MlgImpl

////////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

private object MlgImpl : Mlg {
    override fun check(files: List<File>) {
        TODO("Not yet implemented")
    }

    override fun edit(noOpen: Boolean, port: Int) {
        TODO("Not yet implemented")
    }

    override fun doc() {
        TODO("Not yet implemented")
    }

    override fun clean() {
        TODO("Not yet implemented")
    }

    override fun version() {
        TODO("Not yet implemented")
    }
}
