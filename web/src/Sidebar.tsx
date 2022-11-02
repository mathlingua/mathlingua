import React from 'react';

import { Tree } from 'rsuite';
import 'rsuite/dist/rsuite.css';

import { useFetch } from 'usehooks-ts';
import { Theme, useTheme } from './hooks';
import { PathsResponse } from './types';

export interface SidebarProps {
  onSelect: (path: string) => void;
}

export function Sidebar(props: SidebarProps) {
  const theme = useTheme();
  const styles = getStyles(theme);

  const { data } = useFetch<PathsResponse>('/api/paths');
  const treeData = buildTreeNode(data?.Paths ?? []);

  return (
    <span style={styles.bottom}>
      <Tree
        style={styles.tree}
        data={treeData.children ?? []}
        onSelect={(val) => props.onSelect(val.value as string)}
        renderTreeNode={node => (
          <span style={node.children ? styles.bold : undefined}>
            {(node.label as string ?? '').replace(/_/g, ' ').replace('.math', '')}
          </span>
        )}
      />
    </span>
  );
}

function getStyles(theme: Theme) {
  return {
    top: {
      height: theme.sizeXLarge,
      width: theme.sidebarWidth,
      position: 'fixed',
      left: 0,
      top: 0,
      margin: 0,
      padding: 0,
      borderBottom: 'solid',
      borderBottomColor: theme.gray,
      borderBottomWidth: 1,
    },
    bottom: {
      height: '100%',
      width: theme.sidebarWidth,
      position: 'fixed',
      marginTop: 0,
      marginBottom: 0,
      marginLeft: 0,
      marginRight: 0,
      padding: 0,
      overflow: 'auto',
      borderRight: 'solid',
      borderColor: theme.gray,
      borderWidth: 1,
    },
    tree: {
      height: '100%',
      maxHeight: 'fit-content',
      width: '100%',
      maxWidth: 'fit-content',
    },
    bold: {
      fontWeight: 'bold',
    },
  } as const;
}

interface TreeNode {
  label: string;
  value: string;
  children?: TreeNode[];
}

function buildTreeNode(paths: string[]): TreeNode {
  const sentinal: TreeNode = {
    label: '',
    value: '',
    children: undefined,
  };
  for (const path of paths) {
    populateTreeNode(sentinal, path.split('/'), 0);
  }
  return sentinal;
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
    return node.label === cur;
  });

  let nextRoot: TreeNode;
  if (curIndex >= 0) {
    nextRoot = root.children[curIndex];
  } else {
    nextRoot = {
      label: cur,
      value: parts.slice(0, index+1).join('/'),
      children: undefined,
    };
    root.children.push(nextRoot);
  }

  populateTreeNode(nextRoot, parts, index+1);
}
