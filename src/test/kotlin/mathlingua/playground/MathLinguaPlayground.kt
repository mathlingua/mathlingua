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

package mathlingua.mathlingua.playground

import java.awt.BorderLayout
import java.awt.FlowLayout
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.io.File
import java.io.IOException
import java.lang.StringBuilder
import javax.swing.JButton
import javax.swing.JCheckBox
import javax.swing.JFrame
import javax.swing.JPanel
import javax.swing.JScrollPane
import javax.swing.JSplitPane
import javax.swing.JTextArea
import javax.swing.SwingUtilities
import javax.swing.UIManager
import javax.swing.WindowConstants
import mathlingua.backend.BackEnd
import mathlingua.backend.newSourceCollectionFromContent
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import org.fife.ui.rtextarea.RTextScrollPane

val STATE_FILE = File(System.getProperty("user.home"), ".mathlingua_playground")

fun saveState(text: String) {
    STATE_FILE.writeText(text)
}

fun readState() =
    try {
        STATE_FILE.readText()
    } catch (e: IOException) {
        ""
    }

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
    outputArea.lineWrap = true
    outputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    val inputArea = RSyntaxTextArea(20, 60)
    inputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    inputArea.isCodeFoldingEnabled = true
    inputArea.highlightCurrentLine = false
    inputArea.font = font
    inputArea.syntaxScheme.getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont
    inputArea.text = readState()

    val expandButton = JCheckBox("Expand", true)

    fun processInput() {
        SwingUtilities.invokeLater {
            val collection = newSourceCollectionFromContent(listOf(inputArea.text))
            val errors = BackEnd.check(collection)
            val builder = StringBuilder()
            for (err in errors) {
                builder.append("${err.value.message} (${err.value.row+1}, ${err.value.column+1})\n")
            }

            val expand = expandButton.isSelected

            val htmlPair =
                collection.prettyPrint(
                    input = inputArea.text, html = true, js = "", doExpand = expand)

            val homeDir = File(System.getProperty("user.home"))
            val outputFile = File(homeDir, "mathlingua-playground.html")
            outputFile.writeText(htmlPair.first)
            Runtime.getRuntime().exec("open --background ${outputFile.absolutePath}")

            for (err in htmlPair.second) {
                builder.append("${err.message} (${err.row+1}, ${err.column+1})\n")
            }

            val inputPair =
                collection.prettyPrint(
                    input = inputArea.text, html = false, js = "", doExpand = expand)
            outputArea.text = inputPair.first
            for (err in inputPair.second) {
                builder.append("${err.message} (${err.row+1}, ${err.column+1})\n")
            }

            errorArea.text = builder.toString()
        }
    }

    inputArea.addKeyListener(
        object : KeyListener {
            override fun keyTyped(keyEvent: KeyEvent) {}

            override fun keyReleased(keyEvent: KeyEvent) {
                saveState(inputArea.text)

                if (!keyEvent.isShiftDown || keyEvent.keyCode != KeyEvent.VK_ENTER) {
                    return
                }

                processInput()
            }

            override fun keyPressed(keyEvent: KeyEvent) {}
        })

    val checkButton = JButton("check")
    checkButton.addActionListener { processInput() }

    val bottomPanel = JPanel(FlowLayout(FlowLayout.CENTER))
    bottomPanel.add(checkButton)
    bottomPanel.add(expandButton)

    val inputSplitPane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
    inputSplitPane.leftComponent = RTextScrollPane(inputArea)
    inputSplitPane.rightComponent = JScrollPane(outputArea)
    inputSplitPane.resizeWeight = 0.5

    val textPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    textPane.dividerLocation = 600
    textPane.topComponent = inputSplitPane
    textPane.bottomComponent = JScrollPane(errorArea)

    val mainPanel = JPanel(BorderLayout())
    mainPanel.add(textPane, BorderLayout.CENTER)
    mainPanel.add(bottomPanel, BorderLayout.SOUTH)

    val frame = JFrame()
    frame.setSize(1300, 900)
    frame.contentPane = mainPanel
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.isVisible = true
}
