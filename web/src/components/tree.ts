import { PathLabelPair } from "../types";

export interface TreeNode {
  label: string;
  name: string;
  path: string;
  parent: TreeNode | null;
  children: TreeNode[];
}

export interface BuildTreeNodeResult {
  pathToNode: Map<string, TreeNode>;
  pathToParent: Map<string, TreeNode>;
}

export function buildTreeNode(pairs: PathLabelPair[]): BuildTreeNodeResult {
  const sentinal: TreeNode = {
    label: 'sentinal',
    name: '',
    path: '',
    parent: null,
    children: [],
  };

  const pathToParent = new Map<string, TreeNode>();
  pathToParent.set('', sentinal);

  const pathToNode = new Map<string, TreeNode>();
  pathToNode.set('', sentinal);

  for (const pair of pairs) {
    populateTreeNode(sentinal, pair.Path.split('/'), pair.Label, 0, pathToNode, pathToParent);
  }

  return {
    pathToNode,
    pathToParent,
  };
}

function populateTreeNode(
  root: TreeNode,
  parts: string[],
  label: string,
  index: number,
  pathToNode: Map<string, TreeNode>,
  pathToParent: Map<string, TreeNode>,
) {
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

    pathToNode.set(path, nextRoot);
    pathToParent.set(path, root);
  }

  populateTreeNode(nextRoot, parts, label, index+1, pathToNode, pathToParent);
}
