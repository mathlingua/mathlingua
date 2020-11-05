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

import java.awt.BorderLayout
import java.awt.FlowLayout
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.*
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeModel
import javax.swing.tree.TreePath
import mathlingua.MathLingua
import mathlingua.chalktalk.phase1.ast.Phase1Node
import mathlingua.chalktalk.phase1.ast.getColumn
import mathlingua.chalktalk.phase1.ast.getRow
import mathlingua.chalktalk.phase1.newChalkTalkLexer
import mathlingua.chalktalk.phase1.newChalkTalkParser
import mathlingua.chalktalk.phase2.HtmlCodeWriter
import mathlingua.chalktalk.phase2.ast.Document
import mathlingua.chalktalk.phase2.ast.common.Phase2Node
import mathlingua.chalktalk.phase2.ast.validateDocument
import mathlingua.chalktalk.phase2.findNode
import mathlingua.support.Location
import mathlingua.support.LocationTracker
import mathlingua.support.ValidationFailure
import mathlingua.support.ValidationSuccess
import mathlingua.support.newLocationTracker
import mathlingua.textalk.TexTalkNode
import mathlingua.transform.expandAtNode
import mathlingua.transform.fullExpandComplete
import mathlingua.transform.glueCommands
import mathlingua.transform.moveInlineCommandsToIsNode
import mathlingua.transform.replaceIsNodes
import mathlingua.transform.replaceRepresents
import mathlingua.transform.separateInfixOperatorStatements
import mathlingua.transform.separateIsStatements
import org.fife.ui.rsyntaxtextarea.RSyntaxTextArea
import org.fife.ui.rsyntaxtextarea.SyntaxConstants
import org.fife.ui.rtextarea.RTextScrollPane

fun main() {
    // enable sub-pixel antialiasing
    System.setProperty("awt.useSystemAAFontSettings", "on")
    try {
        UIManager.setLookAndFeel("javax.swing.plaf.nimbus.NimbusLookAndFeel")
    } catch (e: Exception) {
        println("Could not set the look and feel to Nimbus: $e")
    }

    val fontSize = 18
    val fontName = "Courier New"
    val font = Font(fontName, Font.PLAIN, fontSize)
    val boldFont = Font(fontName, Font.BOLD, fontSize)

    val phase1Tree = JTree(DefaultMutableTreeNode())
    val phase2Tree = JTree(DefaultMutableTreeNode())
    val outputTree = JTree(DefaultMutableTreeNode())

    val tokenList = JTextArea(20, 20)
    tokenList.font = font

    val signaturesList = JTextArea(20, 20)
    signaturesList.font = font

    val separateIsBox = JCheckBox("Separate is statements", true)
    val separateInfixOps = JCheckBox("Separate infix operators", true)
    val glueCommands = JCheckBox("Glue commands", true)
    val moveInLineIs = JCheckBox("Move inline is statements", true)
    val replaceReps = JCheckBox("Replace represents", true)
    val replaceIsNodes = JCheckBox("Replace is nodes", true)

    val completeExpand = JCheckBox("Complete expansion", false)
    completeExpand.addActionListener {
        if (completeExpand.isSelected) {
            separateIsBox.isSelected = false
            separateInfixOps.isSelected = false
            glueCommands.isSelected = false
            moveInLineIs.isSelected = false
            replaceReps.isSelected = false
            replaceIsNodes.isSelected = false
        }
    }

    val statusPanel = JPanel(FlowLayout(FlowLayout.LEFT))
    statusPanel.add(separateIsBox)
    statusPanel.add(separateInfixOps)
    statusPanel.add(glueCommands)
    statusPanel.add(moveInLineIs)
    statusPanel.add(replaceReps)
    statusPanel.add(replaceIsNodes)
    statusPanel.add(completeExpand)

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

    val expandButton = JButton("Expand At")
    expandButton.addActionListener {
        val text = inputArea.text
        val validation = MathLingua.parseWithLocations(text)
        if (validation is ValidationSuccess) {
            val doc = validation.value.document
            val tracker = validation.value.tracker

            val lines = inputArea.text.split('\n')
            val offset = inputArea.caretPosition
            var sum = 0
            var row = 0
            while (row < lines.size && sum + lines[row].length + 1 <= offset) {
                sum += lines[row].length + 1
                row++
            }
            val col = offset - sum

            println("row=$row, column=$col")

            val root = toTreeNode(tracker, doc)
            phase2Tree.model = DefaultTreeModel(root)
            val nearestNode = findNode(tracker, doc, row, col)

            println("Found node: $nearestNode")

            val path = getPath(root, nearestNode)
            phase2Tree.expandPath(path)
            phase2Tree.selectionPath = path

            val newDoc = expandAtNode(doc, nearestNode, doc.defines(), doc.states())

            outputArea.text =
                newDoc
                    .toCode(
                        false,
                        0,
                        HtmlCodeWriter(emptyList(), emptyList(), emptyList(), emptyList()))
                    .getCode()
            outputTree.model = DefaultTreeModel(toTreeNode(tracker, newDoc))
        }
    }
    statusPanel.add(expandButton)

    inputArea.addKeyListener(
        object : KeyListener {
            override fun keyTyped(keyEvent: KeyEvent) {}

            override fun keyReleased(keyEvent: KeyEvent) {
                if (!keyEvent.isShiftDown || keyEvent.keyCode != KeyEvent.VK_ENTER) {
                    return
                }

                SwingUtilities.invokeLater {
                    var doc: Document? = null
                    val errorBuilder = StringBuilder()
                    val tracker = newLocationTracker()
                    try {
                        val input = inputArea.text

                        val tmpLexer = newChalkTalkLexer(input)
                        val tokenBuilder = StringBuilder()
                        while (tmpLexer.hasNext()) {
                            val next = tmpLexer.next()
                            val row = next.row
                            val column = next.column
                            tokenBuilder.append("${next.text} <${next.type}>  ($row, $column)\n")
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

                            when (val documentValidation = validateDocument(root, tracker)
                            ) {
                                is ValidationSuccess -> doc = documentValidation.value
                                is ValidationFailure -> {
                                    for ((message, row, column) in documentValidation.errors) {
                                        errorBuilder.append(message)
                                        errorBuilder.append(" (Line: ")
                                        errorBuilder.append(row + 1)
                                        errorBuilder.append(", Column: ")
                                        errorBuilder.append(column + 1)
                                        errorBuilder.append(")\n")
                                    }
                                }
                            }
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
                        for (sig in MathLingua.findAllSignatures(doc, newLocationTracker())) {
                            sigBuilder.append(sig.form)
                            sigBuilder.append('\n')
                        }
                        signaturesList.text = sigBuilder.toString()

                        phase2Tree.model = DefaultTreeModel(toTreeNode(tracker, doc))
                        var transformed = doc

                        if (separateIsBox.isSelected) {
                            transformed =
                                separateIsStatements(transformed, transformed).root as Document
                        }

                        if (separateInfixOps.isSelected) {
                            transformed =
                                separateInfixOperatorStatements(transformed, transformed)
                                    .root as Document
                        }

                        if (glueCommands.isSelected) {
                            transformed = glueCommands(transformed, transformed).root as Document
                        }

                        if (moveInLineIs.isSelected) {
                            transformed =
                                moveInlineCommandsToIsNode(
                                        transformed.defines(), transformed, transformed)
                                    .root as Document
                        }

                        if (replaceReps.isSelected) {
                            transformed =
                                replaceRepresents(transformed, transformed.states(), transformed)
                                    .root as Document
                        }

                        if (replaceIsNodes.isSelected) {
                            transformed =
                                replaceIsNodes(transformed, transformed.defines(), transformed)
                                    .root as Document
                        }

                        if (completeExpand.isSelected) {
                            transformed = fullExpandComplete(doc)
                        }

                        // val translator = LatexTranslator(transformed.defines(),
                        // transformed.represents())
                        // translator.translate(transformed)
                        // outputArea.text = translator.toText()

                        outputArea.text = transformed.toCode(false, 0).getCode()

                        outputTree.model = DefaultTreeModel(toTreeNode(tracker, transformed))
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
    inputSplitPane.leftComponent = RTextScrollPane(inputArea)
    inputSplitPane.rightComponent = JScrollPane(outputArea)
    inputSplitPane.resizeWeight = 0.5

    val textPane = JSplitPane(JSplitPane.VERTICAL_SPLIT)
    textPane.dividerLocation = 600
    textPane.topComponent = inputSplitPane
    textPane.bottomComponent = JScrollPane(errorArea)

    val treeTabbedPane = JTabbedPane()
    treeTabbedPane.addTab("Phase2", JScrollPane(phase2Tree))
    treeTabbedPane.addTab("Phase1", JScrollPane(phase1Tree))
    treeTabbedPane.addTab("Output", JScrollPane(outputTree))
    treeTabbedPane.addTab("Tokens", JScrollPane(tokenList))
    treeTabbedPane.addTab("Signatures", JScrollPane(signaturesList))

    val treePane = JSplitPane(JSplitPane.HORIZONTAL_SPLIT)
    treePane.dividerLocation = 900
    treePane.leftComponent = textPane
    treePane.rightComponent = treeTabbedPane

    val mainPanel = JPanel(BorderLayout())
    mainPanel.add(treePane, BorderLayout.CENTER)
    mainPanel.add(statusPanel, BorderLayout.SOUTH)

    val frame = JFrame()
    frame.setSize(1300, 900)
    frame.contentPane = mainPanel
    frame.defaultCloseOperation = WindowConstants.EXIT_ON_CLOSE
    frame.isVisible = true
}

private fun toTreeNode(phase1Node: Phase1Node): DefaultMutableTreeNode {
    val row = getRow(phase1Node)
    val column = getColumn(phase1Node)
    val result = DefaultMutableTreeNode(phase1Node.javaClass.simpleName + " ($row, $column)")
    var visited = false
    phase1Node.forEach {
        visited = true
        result.add(toTreeNode(it))
    }
    if (!visited) {
        result.add(DefaultMutableTreeNode(phase1Node.toCode() + " ($row, $column)"))
    }
    return result
}

private fun toTreeNode(tracker: LocationTracker, phase2Node: Phase2Node): DefaultMutableTreeNode {
    val result = DefaultMutableTreeNode(Phase2Value(tracker, phase2Node, false))
    var visited = false
    phase2Node.forEach {
        visited = true
        /*
        if (it is Statement) {
            when (it.texTalkRoot) {
                is ValidationSuccess -> {
                    val root = it.texTalkRoot.value
                    result.add(toTreeNode(root))
                }
                is ValidationFailure -> {
                    for (err in it.texTalkRoot.errors) {
                        println(err)
                    }
                    result.add(DefaultMutableTreeNode(it.text))
                }
            }
        } else {
            result.add(toTreeNode(it))
        }
         */
        result.add(toTreeNode(tracker, it))
    }
    if (!visited) {
        result.add(DefaultMutableTreeNode(Phase2Value(tracker, phase2Node, true)))
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

private data class Phase2Value(
    val tracker: LocationTracker, val value: Phase2Node, val showCode: Boolean
) {
    override fun toString(): String {
        val location = tracker.getLocationOf(value) ?: Location(row = -1, column = -1)
        return if (showCode) {
            value.toCode(false, 0).getCode() + " (${location.row}, ${location.column})"
        } else {
            value.javaClass.simpleName + " (${location.row}, ${location.column})"
        }
    }
}

private fun getPath(root: DefaultMutableTreeNode, target: Phase2Node): TreePath {
    val path = mutableListOf<DefaultMutableTreeNode>()
    val found = mutableListOf<MutableList<DefaultMutableTreeNode>>()
    getPathImpl(root, target, path, found)
    val first = found[0]
    val nodes = Array(first.size) { first[it] }
    return TreePath(nodes)
}

private fun getPathImpl(
    root: DefaultMutableTreeNode,
    target: Phase2Node,
    path: MutableList<DefaultMutableTreeNode>,
    found: MutableList<MutableList<DefaultMutableTreeNode>>
) {
    if (path.isNotEmpty() &&
        path.last().userObject is Phase2Value &&
        (path.last().userObject as Phase2Value).value == target) {
        val copy = mutableListOf<DefaultMutableTreeNode>()
        copy.addAll(path)
        found.add(copy)
    }

    path.add(root)
    for (i in 0 until root.childCount) {
        val child = root.getChildAt(i)
        getPathImpl(child as DefaultMutableTreeNode, target, path, found)
    }
    path.removeAt(path.size - 1)
}
