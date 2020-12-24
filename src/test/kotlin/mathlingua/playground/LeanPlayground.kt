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

package mathlingua.mathlingua.playground

import com.beust.klaxon.Klaxon
import java.awt.BorderLayout
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.io.File
import java.util.concurrent.CompletableFuture
import javax.swing.*
import mathlingua.MathLingua
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import org.fife.ui.rtextarea.RTextScrollPane

val BASE_DIR = File(System.getProperty("user.home"), "mlg-playground")

val OUT_FILE = File(BASE_DIR, "out.lean")

var isProcessing = false

data class LeanMessage(
    val caption: String,
    val file_name: String,
    val pos_col: Int,
    val pos_line: Int,
    val severity: String,
    val text: String)

fun main() {
    if (!BASE_DIR.exists()) {
        BASE_DIR.mkdirs()
    }

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

    val inputArea = RSyntaxTextArea(20, 40)
    inputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    inputArea.isCodeFoldingEnabled = true
    inputArea.highlightCurrentLine = false
    inputArea.font = font
    inputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    val inputErrorArea = JTextArea()
    inputErrorArea.font = font

    val outputArea = RSyntaxTextArea(20, 40)
    outputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    outputArea.isCodeFoldingEnabled = true
    outputArea.highlightCurrentLine = false
    outputArea.font = font
    outputArea.lineWrap = true
    outputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    val outputErrorArea = JTextArea()
    outputErrorArea.font = font

    val leftSplitPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    leftSplitPane.topComponent = RTextScrollPane(inputArea)
    leftSplitPane.bottomComponent = JScrollPane(inputErrorArea)
    leftSplitPane.resizeWeight = 0.2

    val rightSplitPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    rightSplitPane.topComponent = RTextScrollPane(outputArea)
    rightSplitPane.bottomComponent = JScrollPane(outputErrorArea)
    rightSplitPane.resizeWeight = 0.2

    val mainSplitPane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
    mainSplitPane.leftComponent = leftSplitPane
    mainSplitPane.rightComponent = rightSplitPane
    mainSplitPane.resizeWeight = 0.5

    val mainPanel = JPanel(BorderLayout())
    mainPanel.add(mainSplitPane, BorderLayout.CENTER)

    val frame = JFrame()
    frame.setSize(1300, 700)
    frame.contentPane = mainPanel
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.isVisible = true

    fun leanParse() {
        if (!isProcessing) {
            isProcessing = true
            CompletableFuture.runAsync {
                outputErrorArea.text = "Processing..."
                OUT_FILE.writeText(outputArea.text)
                val outText =
                    Runtime.getRuntime()
                        .exec("lean --json $OUT_FILE")
                        .inputStream
                        .bufferedReader()
                        .readText()
                isProcessing = false

                SwingUtilities.invokeLater {
                    val text =
                        outText.split("\n").filter { it.trim().isNotEmpty() }.joinToString("\n") {
                            val message = Klaxon().parse<LeanMessage>(it)
                            "${message?.severity}: ${message?.text}"
                        }

                    outputErrorArea.text = text
                }
            }
        }
    }

    inputArea.addKeyListener(
        object : KeyListener {
            override fun keyTyped(e: KeyEvent?) {}

            override fun keyPressed(e: KeyEvent?) {}

            override fun keyReleased(e: KeyEvent?) {
                if (e == null) {
                    return
                }

                inputErrorArea.text = "Processing..."
                when (val validation = MathLingua.parse(inputArea.text)
                ) {
                    is ValidationSuccess -> {
                        // TODO: implement transpiling MathLingua to Lean
                        // outputArea.text = transpiled Lean code
                        // leanParse()
                        inputErrorArea.text = ""
                    }
                    is ValidationFailure -> {
                        val builder = StringBuilder()
                        for (err in validation.errors) {
                            builder.append(
                                "ERROR: ${err.message} (${err.row + 1}, ${err.column + 1})\n\n")
                        }
                        inputErrorArea.text = builder.toString()
                        inputErrorArea.caret.moveDot(0)
                    }
                }
            }
        })

    outputArea.addKeyListener(
        object : KeyListener {
            override fun keyTyped(e: KeyEvent?) {}

            override fun keyPressed(e: KeyEvent?) {}

            override fun keyReleased(e: KeyEvent?) {
                leanParse()
            }
        })
}
