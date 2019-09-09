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
import mathlingua.common.chalktalk.phase1.ast.Phase1Node
import mathlingua.common.chalktalk.phase1.newChalkTalkLexer
import mathlingua.common.chalktalk.phase1.newChalkTalkParser
import mathlingua.common.chalktalk.phase2.Document
import mathlingua.common.chalktalk.phase2.Phase2Node
import mathlingua.common.chalktalk.phase2.Statement
import mathlingua.common.chalktalk.phase2.validateDocument
import mathlingua.common.textalk.Command
import mathlingua.common.textalk.TexTalkNode
import mathlingua.common.transform.getCommandSignature
import mathlingua.common.transform.glueCommands
import mathlingua.common.transform.moveInlineCommandsToIsNode
import mathlingua.common.transform.replaceIsNodes
import mathlingua.common.transform.replaceRepresents
import mathlingua.common.transform.separateIsStatements
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import java.awt.BorderLayout
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.JFrame
import javax.swing.JPanel
import javax.swing.JScrollPane
import javax.swing.JSplitPane
import javax.swing.JTabbedPane
import javax.swing.JTextArea
import javax.swing.JTree
import javax.swing.SwingUtilities
import javax.swing.UIManager
import javax.swing.WindowConstants
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeModel

object TreeViewMain {

    @JvmStatic
    fun main(args: Array<String>) {
        // enable sub-pixel antialiasing
        System.setProperty("awt.useSystemAAFontSettings", "on")
        try {
            UIManager.setLookAndFeel("javax.swing.plaf.nimbus.NimbusLookAndFeel")
        } catch (e: Exception) {
            println("Could not set the look and feel to Nimbus: $e")
        }

        val fontSize = 14
        val fontName = "Brass Mono"
        val font = Font(fontName, Font.PLAIN, fontSize)
        val boldFont = Font(fontName, Font.BOLD, fontSize)

        val phase1Tree = JTree(DefaultMutableTreeNode())
        val phase2Tree = JTree(DefaultMutableTreeNode())

        val tokenList = JTextArea(20, 20)
        tokenList.font = font

        val signaturesList = JTextArea(20, 20)
        signaturesList.font = font

        val errorArea = JTextArea()
        errorArea.font = font

        val outputArea = RSyntaxTextArea(20, 60)
        outputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
        outputArea.isCodeFoldingEnabled = true
        outputArea.highlightCurrentLine = false
        outputArea.font = font
        outputArea.syntaxScheme
            .getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont

        val inputArea = RSyntaxTextArea(20, 60)
        inputArea.syntaxEditingStyle = SyntaxConstants.SYNTAX_STYLE_YAML
        inputArea.isCodeFoldingEnabled = true
        inputArea.highlightCurrentLine = false
        inputArea.font = font
        inputArea.syntaxScheme
            .getStyle(org.fife.ui.rsyntaxtextarea.Token.IDENTIFIER).font = boldFont
        inputArea.addKeyListener(object : KeyListener {
            override fun keyTyped(keyEvent: KeyEvent) {}

            override fun keyReleased(keyEvent: KeyEvent) {
                SwingUtilities.invokeLater {
                    val errorBuilder = StringBuilder()
                    var doc: Document? = null
                    try {
                        val input = inputArea.text

                        val tmpLexer = newChalkTalkLexer(input)
                        val tokenBuilder = StringBuilder()
                        while (tmpLexer.hasNext()) {
                            val next = tmpLexer.next()
                            tokenBuilder.append("${next.text} <${next.type}>\n")
                        }
                        for (err in tmpLexer.errors()) {
                            tokenBuilder.append("ERROR: $err\n")
                        }
                        tokenList.text = tokenBuilder.toString()

                        val lexer = newChalkTalkLexer(input)

                        for (err in lexer.errors()) {
                            errorBuilder.append(err)
                            errorBuilder.append('\n')
                        }

                        val parser = newChalkTalkParser()
                        val (root, errors) = parser.parse(lexer)

                        for (err in errors) {
                            errorBuilder.append(err)
                            errorBuilder.append('\n')
                        }

                        if (root != null) {
                            phase1Tree.model = DefaultTreeModel(toTreeNode(root))
                            val numPhase1Rows = phase1Tree.rowCount
                            if (numPhase1Rows > 0) {
                                phase1Tree.expandRow(numPhase1Rows - 1)
                            }

                            val documentValidation = validateDocument(root)
                            if (!documentValidation.isSuccessful) {
                                for ((message, row, column) in documentValidation.errors) {
                                    errorBuilder.append(message)
                                    errorBuilder.append(" (Line: ")
                                    errorBuilder.append(row + 1)
                                    errorBuilder.append(", Column: ")
                                    errorBuilder.append(column + 1)
                                    errorBuilder.append(")\n")
                                }
                            }

                            doc = documentValidation.value
                        }
                    } catch (e: Exception) {
                        System.err.println(e.message)
                        e.printStackTrace()
                        doc = null
                    }

                    if (doc == null) {
                        outputArea.text = ""
                        signaturesList.text = ""
                        phase2Tree.model = DefaultTreeModel(DefaultMutableTreeNode())
                    } else {
                        val sigBuilder = StringBuilder()
                        val ml = MathLingua()
                        for (sig in ml.findAllSignatures(doc)) {
                            sigBuilder.append(sig)
                            sigBuilder.append('\n')
                        }
                        signaturesList.text = sigBuilder.toString()

                        var transformed = separateIsStatements(doc)
                        transformed = glueCommands(transformed)
                        transformed = moveInlineCommandsToIsNode(doc.defines, transformed, { true }, { true })
                        transformed = replaceRepresents(transformed, doc.represents, { true })
                        transformed = replaceIsNodes(transformed, doc.defines, { true })

                        outputArea.text = transformed.toCode(false, 0)
                        phase2Tree.model = DefaultTreeModel(toTreeNode(doc))
                        val numRows = phase2Tree.rowCount
                        if (numRows > 0) {
                            phase2Tree.expandRow(numRows - 1)
                        }
                    }
                    errorArea.text = errorBuilder.toString()
                }
            }

            override fun keyPressed(keyEvent: KeyEvent) {}
        })

        val inputSplitPane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
        inputSplitPane.leftComponent = JScrollPane(inputArea)
        inputSplitPane.rightComponent = JScrollPane(outputArea)
        inputSplitPane.resizeWeight = 0.5

        val textPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
        textPane.dividerLocation = 600
        textPane.topComponent = inputSplitPane
        textPane.bottomComponent = JScrollPane(errorArea)

        val treeTabbedPane = JTabbedPane()
        treeTabbedPane.addTab("Phase2", JScrollPane(phase2Tree))
        treeTabbedPane.addTab("Phase1", JScrollPane(phase1Tree))
        treeTabbedPane.addTab("Tokens", JScrollPane(tokenList))
        treeTabbedPane.addTab("Signatures", JScrollPane(signaturesList))

        val treePane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
        treePane.dividerLocation = 900
        treePane.leftComponent = textPane
        treePane.rightComponent = treeTabbedPane

        val mainPanel = JPanel(BorderLayout())
        mainPanel.add(treePane, BorderLayout.CENTER)

        val frame = JFrame()
        frame.setSize(1300, 900)
        frame.contentPane = mainPanel
        frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
        frame.isVisible = true
    }

    private fun toTreeNode(phase1Node: Phase1Node): DefaultMutableTreeNode {
        val result = DefaultMutableTreeNode(phase1Node.javaClass.simpleName)
        var visited = false
        phase1Node.forEach {
            visited = true
            result.add(toTreeNode(it))
        }
        if (!visited) {
            result.add(DefaultMutableTreeNode(phase1Node.toCode()))
        }
        return result
    }

    private fun toTreeNode(phase2Node: Phase2Node): DefaultMutableTreeNode {
        val result = DefaultMutableTreeNode(phase2Node.javaClass.simpleName)
        var visited = false
        phase2Node.forEach {
            visited = true
            if (it is Statement) {
                val root = it.texTalkRoot.value
                for (err in it.texTalkRoot.errors) {
                    println(err)
                }
                if (root != null) {
                    result.add(toTreeNode(root))
                } else {
                    result.add(DefaultMutableTreeNode(it.text))
                }
            } else {
                result.add(toTreeNode(it))
            }
        }
        if (!visited) {
            result.add(DefaultMutableTreeNode(phase2Node.toCode(false, 0)))
        }
        return result
    }

    private fun toTreeNode(texTalkNode: TexTalkNode): DefaultMutableTreeNode {
        val result = DefaultMutableTreeNode(texTalkNode.javaClass.simpleName)
        var visited = false
        texTalkNode.forEach {
            visited = true
            result.add(toTreeNode(it))
        }
        if (!visited) {
            result.add(DefaultMutableTreeNode(texTalkNode.toCode()))
        }
        return result
    }
}
