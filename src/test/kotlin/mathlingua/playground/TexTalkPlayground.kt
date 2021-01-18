package mathlingua.mathlingua.playground

import java.awt.BorderLayout
import java.awt.Font
import java.awt.event.KeyEvent
import java.awt.event.KeyListener
import javax.swing.*
import javax.swing.tree.DefaultMutableTreeNode
import javax.swing.tree.DefaultTreeModel
import mathlingua.backend.transform.signature
import mathlingua.frontend.textalk.Command
import mathlingua.frontend.textalk.TexTalkNode
import mathlingua.frontend.textalk.newTexTalkLexer
import mathlingua.frontend.textalk.newTexTalkParser
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

    val outputTree = JTree(DefaultMutableTreeNode())

    val signaturesList = JTextArea(20, 20)
    signaturesList.font = font

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

                        val lexer = newTexTalkLexer(input)

                        for (err in lexer.errors) {
                            errorBuilder.append(err)
                            errorBuilder.append('\n')
                        }

                        val parser = newTexTalkParser()
                        val (root, errors) = parser.parse(lexer)

                        for (err in errors) {
                            errorBuilder.append(err)
                            errorBuilder.append('\n')
                        }

                        outputArea.text = root.toCode()
                        outputTree.model = DefaultTreeModel(toTreeNode(root))

                        val sigBuilder = StringBuilder()
                        for (node in root.children) {
                            if (node is Command) {
                                val sig = node.signature()
                                sigBuilder.append(sig)
                                sigBuilder.append('\n')
                            }
                        }
                        signaturesList.text = sigBuilder.toString()
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

    val treeTabbedPane = JTabbedPane()
    treeTabbedPane.addTab("Output", JScrollPane(outputTree))
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

private fun toTreeNode(texTalkNode: TexTalkNode): DefaultMutableTreeNode {
    val result = DefaultMutableTreeNode(texTalkNode.javaClass.simpleName)
    var visited = false
    texTalkNode.forEach {
        visited = true
        result.add(toTreeNode(it))
    }
    if (!visited) {
        result.add(DefaultMutableTreeNode("${texTalkNode.toCode()} (${texTalkNode.type})"))
    }
    return result
}
