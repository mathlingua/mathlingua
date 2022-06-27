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

package mathlingua

import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.lang.StringBuilder
import javax.swing.JEditorPane
import javax.swing.JFrame
import javax.swing.JScrollPane
import javax.swing.JSplitPane
import javax.swing.JTextArea
import javax.swing.WindowConstants
import mathlingua.lib.frontend.Frontend

fun main() {
    val font = Font("Courier New", Font.PLAIN, 22)

    val errorPane = JEditorPane()
    errorPane.font = font
    errorPane.isEditable = false

    val editor = JTextArea()
    editor.font = font
    editor.addKeyListener(
        object : KeyListener {
            override fun keyTyped(e: KeyEvent?) {
                errorPane.text = ""
                val result = Frontend.parse(editor.text)
                val builder = StringBuilder()
                for (diag in result.diagnostics) {
                    builder.append(
                        "${diag.type}: ${diag.message} (${diag.row + 1}, ${diag.column + 1}) @${diag.origin}\n")
                }
                errorPane.text = builder.toString()
            }

            override fun keyPressed(e: KeyEvent?) {}

            override fun keyReleased(e: KeyEvent?) {}
        })

    val splitPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    splitPane.dividerLocation = 750
    splitPane.topComponent = JScrollPane(editor)
    splitPane.bottomComponent = JScrollPane(errorPane)

    val frame = JFrame("ChalkTalk Testbed")
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.contentPane = splitPane
    frame.setSize(1000, 1000)
    frame.isVisible = true
}
