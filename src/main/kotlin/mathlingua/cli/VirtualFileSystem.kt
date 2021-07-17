/*
 * Copyright 2021
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

package mathlingua.cli

class VirtualFileException(message: String) : Exception(message)

interface VirtualFile {
    fun absolutePath(): List<String>
    fun relativePathTo(dir: VirtualFile): List<String>
    fun isDirectory(): Boolean
    fun exists(): Boolean
    fun readText(): String
    fun writeText(text: String)
    fun mkdirs(): Boolean
    fun listFiles(): List<VirtualFile>
    fun delete(): Boolean
}

interface VirtualFileSystem {
    fun getFileSeparator(): String
    fun getFileOrDirectory(path: String): VirtualFile
    fun getFile(relativePath: List<String>): VirtualFile
    fun getDirectory(relativePath: List<String>): VirtualFile
    fun cwd(): VirtualFile
    fun relativePathTo(vf: VirtualFile, dir: VirtualFile): List<String>
    fun exists(vf: VirtualFile): Boolean
    fun mkdirs(vf: VirtualFile): Boolean
    fun readText(vf: VirtualFile): String
    fun writeText(vf: VirtualFile, content: String)
    fun listFiles(vf: VirtualFile): List<VirtualFile>
    // deletes the virtual file if it is a file and recursively
    // if it is a directory
    fun delete(vf: VirtualFile): Boolean
}

fun newMemoryFileSystem(cwd: List<String>): VirtualFileSystem {
    return MemoryFileSystem(cwd)
}

data class VirtualFileImpl(
    private val absolutePathParts: List<String>,
    private val directory: Boolean,
    private val fs: VirtualFileSystem
) : VirtualFile {

    override fun absolutePath() = absolutePathParts

    override fun relativePathTo(dir: VirtualFile) = fs.relativePathTo(this, dir)

    override fun isDirectory() = directory

    override fun exists() = fs.exists(this)

    override fun readText() = fs.readText(this)

    override fun writeText(text: String) = fs.writeText(this, text)

    override fun mkdirs() = fs.mkdirs(this)

    override fun listFiles() = fs.listFiles(this)

    override fun delete() = fs.delete(this)

    override fun toString() = absolutePath().joinToString(fs.getFileSeparator())
}

private class MemoryFileNode(
    val name: String,
    val isDirectory: Boolean,
    var content: String,
    val children: MutableMap<String, MemoryFileNode>)

private class MemoryFileSystem(private val cwd: List<String>) : VirtualFileSystem {

    private val root =
        MemoryFileNode(name = "", isDirectory = true, content = "", children = mutableMapOf())

    private val cwdFile = VirtualFileImpl(absolutePathParts = cwd, directory = true, fs = this)

    init {
        var tail = root
        for (part in cwd) {
            val child =
                MemoryFileNode(
                    name = part, isDirectory = true, content = "", children = mutableMapOf())
            tail.children[part] = child
            tail = child
        }
    }

    private fun getAbsolutePath(relativePath: List<String>): List<String> {
        val absolutePath = mutableListOf<String>()
        absolutePath.addAll(cwd)
        absolutePath.addAll(relativePath)
        return absolutePath
    }

    override fun getFileSeparator() = "/"

    override fun getFileOrDirectory(path: String) =
        if (path.endsWith(getFileSeparator())) {
            getDirectory(path.split(getFileSeparator()))
        } else {
            getFile(path.split(getFileSeparator()))
        }

    override fun getFile(relativePath: List<String>): VirtualFile {
        return VirtualFileImpl(
            absolutePathParts = getAbsolutePath(relativePath), directory = false, fs = this)
    }

    override fun getDirectory(relativePath: List<String>): VirtualFile {
        return VirtualFileImpl(
            absolutePathParts = getAbsolutePath(relativePath), directory = true, fs = this)
    }

    override fun cwd() = cwdFile

    override fun relativePathTo(vf: VirtualFile, dir: VirtualFile): List<String> {
        val vfParts = vf.absolutePath()
        val dirParts = dir.absolutePath()
        var i = 0
        while (i < dirParts.size && i < vfParts.size && dirParts[i] == vfParts[i]) {
            i++
        }
        val relParts = mutableListOf<String>()
        while (i < vfParts.size) {
            relParts.add(vfParts[i])
        }
        return relParts
    }

    override fun exists(vf: VirtualFile): Boolean {
        var tail = root
        for (part in vf.absolutePath()) {
            if (!tail.children.containsKey(part)) {
                return false
            }
            tail = tail.children[part]!!
        }
        return true
    }

    override fun mkdirs(vf: VirtualFile): Boolean {
        var tail = root
        for (part in vf.absolutePath()) {
            if (!tail.children.containsKey(part)) {
                val child =
                    MemoryFileNode(
                        name = part, isDirectory = true, content = "", children = mutableMapOf())
                tail.children[part] = child
                tail = child
            } else {
                val child = tail.children[part]!!
                if (!child.isDirectory) {
                    return false
                }
            }
        }
        return true
    }

    private fun getMemoryNode(absolutePath: List<String>, createAsFile: Boolean): MemoryFileNode {
        var tail = root
        for (part in absolutePath) {
            if (!tail.children.containsKey(part)) {
                if (createAsFile) {
                    val newFile =
                        MemoryFileNode(
                            name = part,
                            isDirectory = false,
                            content = "",
                            children = mutableMapOf())
                    tail.children[part] = newFile
                    tail = newFile
                } else {
                    throw VirtualFileException(
                        "File not found: ${absolutePath.joinToString(getFileSeparator())}")
                }
            } else {
                tail = tail.children[part]!!
            }
        }
        return tail
    }

    override fun readText(vf: VirtualFile) =
        getMemoryNode(vf.absolutePath(), createAsFile = false).content

    override fun writeText(vf: VirtualFile, content: String) {
        getMemoryNode(vf.absolutePath(), createAsFile = true).content = content
    }

    override fun listFiles(vf: VirtualFile): List<VirtualFile> {
        val node = getMemoryNode(vf.absolutePath(), createAsFile = false)
        return node.children.values.map {
            val parts = mutableListOf<String>()
            parts.addAll(vf.absolutePath())
            parts.add(it.name)
            VirtualFileImpl(absolutePathParts = parts, directory = it.isDirectory, fs = this)
        }
    }

    override fun delete(vf: VirtualFile): Boolean {
        val path = vf.absolutePath()
        val parentPath = path.filterIndexed { index, _ -> index < path.size - 1 }
        val parent = getMemoryNode(parentPath, createAsFile = true)
        val name = path.last()
        parent.children.remove(name)
        return true
    }
}
