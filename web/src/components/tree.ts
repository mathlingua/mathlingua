import { PathLabelPair } from "../types";

export interface TreeNode {
  label: string;
  name: string;
  path: string;
  parent: TreeNode | null;
  children: TreeNode[];
}

export function buildTreeNode(pairs: PathLabelPair[]): { tree: TreeNode; } {
  const sentinal: TreeNode = {
    label: '',
    name: '',
    path: '',
    parent: null,
    children: [],
  };
  for (const pair of pairs) {
    populateTreeNode(sentinal, pair.Path.split('/'), pair.Label, 0);
  }
  return {
    tree: sentinal,
  };
}

function populateTreeNode(root: TreeNode, parts: string[], label: string, index: number) {
  if (index >= parts.length) {
    return;
  }

  if (root.children === undefined) {
    root.children = [];
  }

  const cur = parts[index]!;
  const curIndex = root.children.findIndex((node) => {
    return node.name === cur;
  });

  let nextRoot: TreeNode;
  if (curIndex >= 0) {
    nextRoot = root.children[curIndex];
  } else {
    const path = parts.slice(0, index+1).join('/');
    nextRoot = {
      label: index === parts.length - 1 ? label : cur,
      name: cur,
      path,
      parent: root,
      children: [],
    };
    root.children.push(nextRoot);
  }

  populateTreeNode(nextRoot, parts, label, index+1);
}
