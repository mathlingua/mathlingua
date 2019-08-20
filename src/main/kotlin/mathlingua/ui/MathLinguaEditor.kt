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

package mathlingua.ui

import mathlingua.common.MathLingua
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import org.fife.ui.rtextarea.RTextScrollPane
import java.awt.BorderLayout
import java.awt.Color
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import java.io.FileNotFoundException
import java.io.IOException
import java.nio.file.Paths
import javax.swing.*

val SOURCE_FILE = Paths.get("src", "main", "resources", "mathlingua.txt").toFile()

object MathLinguaEditor {
  @JvmStatic
  fun main(args: Array<String>) {
    // enable sub-pixel antialiasing
    System.setProperty("awt.useSystemAAFontSettings", "on")
    try {
      UIManager.setLookAndFeel("javax.swing.plaf.nimbus.NimbusLookAndFeel")
    } catch (e: Exception) {
      println("Could not set the look and feel to Nimbus: $e")
    }

    val fontSize = 26
    val fontName = "Brass Mono"
    val font = Font(fontName, Font.PLAIN, fontSize)
    val boldFont = Font(fontName, Font.BOLD, fontSize)

    val errorArea = JTextArea()
    errorArea.font = font
    errorArea.foreground = Color.RED

    val inputArea = RSyntaxTextArea(20, 60)
    inputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
    inputArea.isCodeFoldingEnabled = true
    inputArea.highlightCurrentLine = false
    inputArea.font = font
    inputArea.syntaxScheme
      .getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

    val scrollPane = RTextScrollPane(inputArea)
    scrollPane.lineNumbersEnabled = true
    scrollPane.isFoldIndicatorEnabled = false

    val textPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    textPane.dividerLocation = 600
    textPane.topComponent = scrollPane
    textPane.bottomComponent = JScrollPane(errorArea)

    val mainPanel = JPanel(BorderLayout())
    mainPanel.add(textPane, BorderLayout.CENTER)

    val frame = JFrame()
    frame.setSize(1300, 900)
    frame.contentPane = mainPanel
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.isVisible = true

    try {
      inputArea.text = String(SOURCE_FILE.readBytes())
    } catch (e: FileNotFoundException) {
      // don't worry if the source file does not exist
    }

    inputArea.addKeyListener(object : KeyListener {
      override fun keyTyped(keyEvent: KeyEvent) {}

      override fun keyReleased(keyEvent: KeyEvent) {
        SwingUtilities.invokeLater {
          if (!keyEvent.isControlDown || keyEvent.keyCode != KeyEvent.VK_S) {
            frame.title = "Modified"
          }

          val result = MathLingua.parse(inputArea.text)
          val errorBuilder = StringBuilder()
          for (err in result.errors) {
            errorBuilder.append("Error(${err.row + 1}, ${err.column + 1}): ${err.message}\n")
          }
          errorArea.text = errorBuilder.toString()
        }
      }

      override fun keyPressed(keyEvent: KeyEvent) {
        if (keyEvent.isControlDown && keyEvent.keyCode == KeyEvent.VK_S) {
          SwingUtilities.invokeLater {
            frame.title = "Saving"
            try {
              SOURCE_FILE.writeBytes(inputArea.text.toByteArray())
              frame.title = "Saved"
            } catch (e: IOException) {
              frame.title = "Failed to save: ${e.message}"
            }
          }
        }
      }
    })
  }
}
