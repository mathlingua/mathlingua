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

package mathlingua.mathlingua.playground

import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.JFrame
import javax.swing.JScrollPane
import javax.swing.JSplitPane
import javax.swing.JTextArea
import javax.swing.SwingUtilities
import javax.swing.UIManager
import javax.swing.WindowConstants
import mathlingua.MathLingua
import mathlingua.frontend.support.ValidationFailure
import mathlingua.frontend.support.ValidationSuccess
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import org.fife.ui.rtextarea.RTextScrollPane

fun main() {
    // enable sub-pixel antialiasing
    System.setProperty("awt.useSystemAAFontSettings", "on")
    try {
        UIManager.setLookAndFeel(UIManager.getSystemLookAndFeelClassName())
    } catch (e: Exception) {
        println("Could not set the system look and feel: $e")
    }

    val fontSize = 18
    val fontName = "Courier New"
    val font = Font(fontName, Font.PLAIN, fontSize)
    val boldFont = Font(fontName, Font.BOLD, fontSize)

    val errorArea = JTextArea()
    errorArea.font = font

    val outputArea = RSyntaxTextArea(20, 60)
    outputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    outputArea.isCodeFoldingEnabled = true
    outputArea.highlightCurrentLine = false
    outputArea.font = font
    outputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    val inputArea = RSyntaxTextArea(20, 60)
    inputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    inputArea.isCodeFoldingEnabled = true
    inputArea.highlightCurrentLine = false
    inputArea.font = font
    inputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    inputArea.addKeyListener(
        object : KeyListener {
            override fun keyTyped(keyEvent: KeyEvent) {}

            override fun keyReleased(keyEvent: KeyEvent) {
                if (!keyEvent.isShiftDown || keyEvent.keyCode != KeyEvent.VK_ENTER) {
                    return
                }

                SwingUtilities.invokeLater {
                    val errorBuilder = StringBuilder()
                    try {
                        val input = inputArea.text
                        outputArea.text = ""
                        when (val validation = MathLingua.printExpanded(input, input, false)
                        ) {
                            is ValidationSuccess -> outputArea.text = validation.value
                            is ValidationFailure -> {
                                for (err in validation.errors) {
                                    errorBuilder.append(
                                        "${err.message} (${err.row}, ${err.column})")
                                }
                            }
                        }
                    } catch (e: Exception) {
                        System.err.println(e.message)
                        e.printStackTrace()
                    }
                    errorArea.text = errorBuilder.toString()
                }
            }

            override fun keyPressed(keyEvent: KeyEvent) {}
        })

    val inputSplitPane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
    inputSplitPane.leftComponent = RTextScrollPane(inputArea)
    inputSplitPane.rightComponent = JScrollPane(outputArea)
    inputSplitPane.resizeWeight = 0.5

    val textPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    textPane.dividerLocation = 600
    textPane.topComponent = inputSplitPane
    textPane.bottomComponent = JScrollPane(errorArea)

    val frame = JFrame()
    frame.setSize(1300, 900)
    frame.contentPane = textPane
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.isVisible = true
}
