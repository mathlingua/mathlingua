export class AutoComplete {
  private root = new TrieNode(false);

  add(word: string) {
    this.addToTrie(this.root, word, 0);
  }

  getSuffixes(prefix: string) {
    return this.searchTrie(this.root, prefix);
  }

  private addToTrie(trieNode: TrieNode, word: string, index: number) {
    if (index >= word.length) {
      return;
    }

    const c = word[index];
    if (!trieNode.children.has(c)) {
      trieNode.children.set(c, {
        isWord: index === word.length - 1,
        children: new Map(),
      });
    }

    const subNode = trieNode.children.get(c)!;
    if (index === word.length - 1) {
      subNode.isWord = true;
    }

    this.addToTrie(subNode, word, index + 1);
  }

  searchTrie(trieNode: TrieNode, word: string) {
    const node = this.findTrieLeaf(trieNode, word, 0);
    if (!node) {
      return [];
    }
    const result = new Set<string>();
    this.getWordsUnder('', '', node, result);
    return Array.from(result);
  }

  findTrieLeaf(
    trieNode: TrieNode,
    word: string,
    index: number
  ): TrieNode | null {
    if (index >= word.length) {
      return trieNode;
    }

    const c = word[index];
    if (trieNode.children.has(c)) {
      return this.findTrieLeaf(trieNode.children.get(c)!, word, index + 1);
    }

    return null;
  }

  getWordsUnder(
    buffer: string,
    char: string,
    trieNode: TrieNode,
    result: Set<string>
  ) {
    if (!trieNode) {
      return;
    }

    buffer += char;
    if (trieNode.isWord) {
      result.add(buffer);
    }

    for (const c of trieNode.children.keys() as any) {
      this.getWordsUnder(buffer, c, trieNode.children.get(c)!, result);
    }

    buffer = buffer.substring(0, buffer.length - 1);
  }
}

class TrieNode {
  isWord: boolean;
  readonly children: Map<string, TrieNode>;

  constructor(isWord: boolean) {
    this.isWord = isWord;
    this.children = new Map();
  }
}
