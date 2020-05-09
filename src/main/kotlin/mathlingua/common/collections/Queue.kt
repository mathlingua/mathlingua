/*
 * Copyright 2020 Google LLC
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

package mathlingua.common.collections

interface Queue<T> : Iterable<T> {
    fun offer(value: T)
    fun poll(): T
    fun peek(): T
    fun isEmpty(): Boolean
}

fun <T> newQueue(): Queue<T> = QueueImpl()

// ------------------------------------------------------------------------------------------------------------------ //

private class QueueImpl<T> : Queue<T> {
    private val data = mutableListOf<T>()

    override fun offer(value: T) {
        data.add(0, value)
    }

    override fun poll() = data.removeAt(data.size - 1)

    override fun peek() = data.last()

    override fun isEmpty() = data.isEmpty()

    override fun iterator() = data.iterator()
}
