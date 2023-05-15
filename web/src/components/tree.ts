
export interface TreeNode {
  label: string;
  name: string;
  path: string;
  parent: TreeNode | null;
  children: TreeNode[];
}

export function buildTreeNode(paths: string[]): { tree: TreeNode; } {
  const sentinal: TreeNode = {
    label: '',
    name: '',
    path: '',
    parent: null,
    children: [],
  };
  for (const path of paths) {
    populateTreeNode(sentinal, path.split('/'), 0);
  }
  return {
    tree: sentinal,
  };
}

function populateTreeNode(root: TreeNode, parts: string[], index: number) {
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
      label: formatName(cur),
      name: cur,
      path,
      parent: root,
      children: [],
    };
    root.children.push(nextRoot);
  }

  populateTreeNode(nextRoot, parts, index+1);
}

function formatName(name: string): string {
  const text = name.replace(/\.math$/g, '').replace(/_/g, ' ');
  if (text.length === 0) {
    return '';
  }
  return text.split(' ').map(part => {
    const first = part[0];
    const tail = part.substring(1);
    return first.toUpperCase() + tail;
  }).join(' ');
}
